use reqwest::header::HeaderMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::{sleep_until, Instant};

#[derive(Debug, Clone)]
pub struct HeaderMetaData {
    pub ratelimiter: RateLimiter,
    pub total_page: Arc<Mutex<u64>>,
}

impl HeaderMetaData {
    pub fn new(ratelimiter: RateLimiter) -> Self {
        Self {
            ratelimiter,
            total_page: Arc::new(Mutex::new(u64::MAX)),
        }
    }

    pub fn update_from_headers(&self, headers: &HeaderMap) {
        let parse_u64 = |name: &str| -> Option<u64> {
            headers
                .get(name)
                .and_then(|v| v.to_str().ok())?
                .parse()
                .ok()
        };

        if let Some(total) = parse_u64("x-total") {
            *self.total_page.lock().unwrap() = total;
        }
        self.ratelimiter.update_from_headers(headers);
    }
}

#[derive(Debug)]
struct Inner {
    sec_limit: u64,
    hour_limit: u64,
    sec_remaining: u64,
    hour_remaining: u64,
    sec_reset: Instant,
    hour_reset: Instant,
    retry_after_until: Option<Instant>,
}

#[derive(Debug, Clone)]
pub struct RateLimiter {
    inner: Arc<Mutex<Inner>>,
}

impl RateLimiter {
    pub fn new(per_second_limit: u64, hourly_limit: u64) -> Self {
        let now = Instant::now();
        let inner = Inner {
            sec_limit: per_second_limit,
            hour_limit: hourly_limit,
            sec_remaining: per_second_limit,
            hour_remaining: hourly_limit,
            sec_reset: now + Duration::from_secs(1),
            hour_reset: now + Duration::from_secs(3600),
            retry_after_until: None,
        };
        Self {
            inner: Arc::new(Mutex::new(inner)),
        }
    }

    /// 헤더 기반 갱신: 한 번만 락 잡고 끝냄
    pub fn update_from_headers(&self, headers: &HeaderMap) {
        let parse_u64 = |name: &str| -> Option<u64> {
            headers
                .get(name)
                .and_then(|v| v.to_str().ok())?
                .parse()
                .ok()
        };

        let mut st = self.inner.lock().unwrap();

        if let Some(rem) = parse_u64("x-secondly-ratelimit-remaining") {
            // 서버가 알려준 값으로 덮어써서 동기화
            st.sec_remaining = rem.min(st.sec_limit);
        }
        if let Some(rem) = parse_u64("x-hourly-ratelimit-remaining") {
            st.hour_remaining = rem.min(st.hour_limit);
        }
        if let Some(secs) = parse_u64("retry-after") {
            st.retry_after_until = Some(Instant::now() + Duration::from_secs(secs));
        }
    }

    /// 요청 전 호출: 락은 매우 짧게만 잡고, 대기는 락 밖에서 수행
    pub async fn acquire(&self) {
        loop {
            // 락을 짧게 잡아서 '무엇을 할지'만 결정하고 곧바로 풀기
            let decision = {
                let mut st = self.inner.lock().unwrap();
                let now = Instant::now();

                // 1) Retry-After가 남아있으면 그 시각까지 잔다
                if let Some(deadline) = st.retry_after_until {
                    if now < deadline {
                        Control::Sleep(deadline)
                    } else {
                        st.retry_after_until = None; // 만료됨
                        Control::Recheck
                    }
                } else {
                    // 2) 윈도 리셋
                    if now >= st.sec_reset {
                        st.sec_remaining = st.sec_limit;
                        st.sec_reset = now + Duration::from_secs(1);
                    }
                    if now >= st.hour_reset {
                        st.hour_remaining = st.hour_limit;
                        st.hour_reset = now + Duration::from_secs(3600);
                    }

                    // 3) 토큰 소비 가능?
                    if st.sec_remaining > 0 && st.hour_remaining > 0 {
                        st.sec_remaining -= 1;
                        st.hour_remaining -= 1;
                        Control::Permit
                    } else {
                        // 부족한 쪽의 리셋 시각까지 잔다
                        let next = if st.sec_remaining == 0 {
                            st.sec_reset
                        } else {
                            st.hour_reset
                        };
                        Control::Sleep(next)
                    }
                }
            };

            match decision {
                Control::Permit => return, // 바로 진행
                Control::Sleep(deadline) => sleep_until(deadline).await,
                Control::Recheck => {} // 즉시 루프 재검사
            }
        }
    }
}

enum Control {
    Permit,
    Sleep(Instant),
    Recheck,
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::header::{HeaderMap, HeaderValue};
    use tokio::time as ttime;
    use ttime::{Duration, Instant};

    // ---- 위에 붙여둔 with_windows 도우미가 이 모듈 안에 함께 있어야 합니다. ----
    #[cfg(any(test, feature = "test_helpers"))]
    impl RateLimiter {
        fn with_windows(
            per_second_limit: u64,
            hourly_limit: u64,
            sec_window: std::time::Duration,
            hour_window: std::time::Duration,
        ) -> Self {
            let now = Instant::now();
            let inner = Inner {
                sec_limit: per_second_limit,
                hour_limit: hourly_limit,
                sec_remaining: per_second_limit,
                hour_remaining: hourly_limit,
                sec_reset: now + Duration::from_secs_f64(sec_window.as_secs_f64()),
                hour_reset: now + Duration::from_secs_f64(hour_window.as_secs_f64()),
                retry_after_until: None,
            };
            Self {
                inner: std::sync::Arc::new(std::sync::Mutex::new(inner)),
            }
        }
    }

    /// 제한 이내에서는 대기 없이 통과
    #[tokio::test(start_paused = true)]
    async fn test_can_acquire_within_limit() {
        let limiter =
            RateLimiter::with_windows(5, 100, Duration::from_secs(1), Duration::from_secs(3600));
        let t0 = Instant::now();
        for _ in 0..5 {
            limiter.acquire().await;
        }
        // 가상 시간은 전진하지 않음(슬립이 없었단 뜻)
        assert_eq!(Instant::now() - t0, Duration::from_millis(0));
    }

    /// 초당 제한 초과 시 다음 윈도우까지 정확히 대기
    #[tokio::test(start_paused = true)]
    async fn test_waits_when_per_second_limit_exceeded() {
        let limiter =
            RateLimiter::with_windows(3, 100, Duration::from_secs(1), Duration::from_secs(3600));
        // 3개는 즉시
        for _ in 0..3 {
            limiter.acquire().await;
        }
        // 4번째는 1초 뒤
        let j = tokio::spawn({
            let l = limiter.clone();
            async move { l.acquire().await }
        });

        ttime::advance(Duration::from_millis(999)).await;
        assert!(!j.is_finished(), "아직 1초 미만이므로 완료되면 안됨");

        ttime::advance(Duration::from_millis(1)).await; // 총 1s
        j.await.unwrap();
    }

    /// 짧은 윈도우(200ms)에서 리셋 확인
    #[tokio::test(start_paused = true)]
    async fn test_limit_resets_after_short_window() {
        let limiter = RateLimiter::with_windows(
            2,
            100,
            Duration::from_millis(200),
            Duration::from_secs(3600),
        );
        limiter.acquire().await;
        limiter.acquire().await;

        let j = tokio::spawn({
            let l = limiter.clone();
            async move { l.acquire().await }
        });

        ttime::advance(Duration::from_millis(199)).await;
        assert!(!j.is_finished(), "아직 200ms 전이므로 대기해야 함");

        ttime::advance(Duration::from_millis(1)).await; // 200ms 도달
        j.await.unwrap();
    }

    /// 동시 접근 시 초당 한 배치씩 처리되는지(배칭) 확인
    #[tokio::test(start_paused = true)]
    async fn test_concurrent_acquires_batching() {
        let limiter =
            RateLimiter::with_windows(8, 1000, Duration::from_secs(1), Duration::from_secs(3600));

        let mut handles = Vec::new();
        for _ in 0..32 {
            let l = limiter.clone();
            handles.push(tokio::spawn(async move { l.acquire().await }));
        }

        // 스케줄링 기회 부여
        tokio::task::yield_now().await;
        assert_eq!(
            handles.iter().filter(|h| h.is_finished()).count(),
            8,
            "첫 8개는 즉시 통과"
        );

        for i in 1..=3 {
            ttime::advance(Duration::from_secs(1)).await;
            tokio::task::yield_now().await;
            assert_eq!(
                handles.iter().filter(|h| h.is_finished()).count(),
                8 * (i + 1),
                "매 1초마다 8개씩 완료되어야 함"
            );
        }

        for h in handles {
            h.await.unwrap();
        }
    }

    /// retry-after 헤더가 다음 acquire를 정확히 지연
    #[tokio::test(start_paused = true)]
    async fn test_retry_after_delays_acquire() {
        let limiter =
            RateLimiter::with_windows(5, 100, Duration::from_secs(1), Duration::from_secs(3600));
        let mut headers = HeaderMap::new();
        headers.insert("retry-after", HeaderValue::from_static("2"));
        limiter.update_from_headers(&headers);

        let j = tokio::spawn({
            let l = limiter.clone();
            async move { l.acquire().await }
        });

        ttime::advance(Duration::from_secs(1)).await;
        assert!(!j.is_finished(), "2초 이전이므로 대기 중이어야 함");

        ttime::advance(Duration::from_secs(1)).await; // 총 2s
        j.await.unwrap();
    }

    /// 서버가 secondly remaining=0을 보냈을 때 즉시 대기 시작하는지
    #[tokio::test(start_paused = true)]
    async fn test_header_remaining_zero_enforces_wait() {
        let limiter = RateLimiter::with_windows(
            2,
            100,
            Duration::from_millis(300),
            Duration::from_secs(3600),
        );

        let mut headers = HeaderMap::new();
        headers.insert(
            "x-secondly-ratelimit-remaining",
            HeaderValue::from_static("0"),
        );
        limiter.update_from_headers(&headers);

        let j = tokio::spawn({
            let l = limiter.clone();
            async move { l.acquire().await }
        });

        ttime::advance(Duration::from_millis(299)).await;
        assert!(!j.is_finished());
        ttime::advance(Duration::from_millis(1)).await; // 300ms 경과
        j.await.unwrap();
    }

    /// (테스트용) 시간당 윈도우를 2초로 줄여서 hour limit 동작 확인
    #[tokio::test(start_paused = true)]
    async fn test_hourly_window_respected_with_short_window() {
        // 초당은 넉넉(100), 시간당은 3, hour_window 2초
        let limiter =
            RateLimiter::with_windows(100, 3, Duration::from_millis(50), Duration::from_secs(2));

        for _ in 0..3 {
            limiter.acquire().await; // 시간당 3개 소진
        }

        let j = tokio::spawn({
            let l = limiter.clone();
            async move { l.acquire().await } // 4번째 -> hour reset까지 대기
        });

        ttime::advance(Duration::from_secs(1)).await;
        assert!(!j.is_finished(), "아직 hour 윈도우(2s) 전");

        ttime::advance(Duration::from_secs(1)).await; // 총 2s -> 리셋
        j.await.unwrap();
    }

    /// retry-after > per-second reset: 더 강한 제약이 우선하는지 확인
    #[tokio::test(start_paused = true)]
    async fn test_interleaved_retry_after_and_window() {
        // 초당 2개, 300ms 윈도우
        let limiter = RateLimiter::with_windows(
            2,
            100,
            Duration::from_millis(300),
            Duration::from_secs(3600),
        );

        // 첫 토큰 소비
        limiter.acquire().await;

        // 서버가 1초 retry-after 지시
        let mut headers = HeaderMap::new();
        headers.insert("retry-after", HeaderValue::from_static("1"));
        limiter.update_from_headers(&headers);

        // 다음 acquire는 retry-after 끝까지 대기해야 함
        let j = tokio::spawn({
            let l = limiter.clone();
            async move { l.acquire().await }
        });

        // per-second 윈도우가 먼저 지나가도…
        ttime::advance(Duration::from_millis(300)).await;
        tokio::task::yield_now().await;
        assert!(!j.is_finished(), "윈도우 리셋이 와도 retry-after가 우선");

        // retry-after 종료 시 진행
        ttime::advance(Duration::from_millis(700)).await; // 총 1s
        j.await.unwrap();
    }

    /// HeaderMetaData가 x-total을 반영하는지(부가 메타 확인)
    #[test]
    fn test_header_metadata_updates_total_page() {
        let meta = HeaderMetaData::new(RateLimiter::new(5, 100));
        let mut headers = HeaderMap::new();
        headers.insert("x-total", HeaderValue::from_static("42"));
        meta.update_from_headers(&headers);

        let total = *meta.total_page.lock().unwrap();
        assert_eq!(total, 42);
    }
}
