use std::{ops::ControlFlow, sync::Arc, time::Duration};

use crate::prelude::*;

use futures::future::BoxFuture;
use tokio::time::sleep;

pub fn req_validator<F, RS>(f: F) -> F
where
    F: for<'a> Fn(
        Arc<FtClientSession<'a, FtClientReqwestConnector>>,
        usize,
    ) -> BoxFuture<'a, ClientResult<RS>>,
{
    f
}
pub type ReqFn<RS> = for<'a> fn(
    Arc<FtClientSession<'a, FtClientReqwestConnector>>,
    usize,
) -> BoxFuture<'a, ClientResult<RS>>;

pub async fn scroller<'a, T, RS, RQ>(
    client: &'a FtClient<FtClientReqwestConnector>,
    thread_num: usize,
    initial_page: usize,
    request_builder: RQ,
) -> Vec<T>
where
    RS: for<'de> serde::de::Deserialize<'de> + HasVec<T>,
    RQ: Fn(
        Arc<FtClientSession<'a, FtClientReqwestConnector>>,
        usize,
    ) -> BoxFuture<'a, ClientResult<RS>>,
{
    let mut result = Vec::new();
    let token = FtApiToken::try_get(AuthInfo::build_from_env().unwrap())
        .await
        .unwrap();
    let session = Arc::new(client.open_session(token));
    let request = Arc::new(request_builder);

    let mut page = initial_page;
    while *client.meta.total_page.lock().unwrap() as usize >= page {
        let page = &mut page;
        let request = Arc::clone(&request);
        if let ControlFlow::Break(()) = {
            let result = &mut result;
            let session_clone = Arc::clone(&session);
            async move {
                let res = request(session_clone, *page).await;
                match res {
                    Ok(res) => {
                        if res.get_vec().is_empty() {
                            return ControlFlow::Break(());
                        }

                        result.extend(res.take_vec());
                        *page += thread_num;
                    }
                    Err(FtClientError::RateLimitError(_)) => {
                        tracing::warn!("rate limit, try again.");
                        sleep(Duration::new(1, 42)).await
                    }
                    Err(e) => {
                        eprintln!("other error: {e}");
                        return ControlFlow::Break(());
                    }
                }
                ControlFlow::Continue(())
            }
        }
        .await
        {
            break;
        }
    }
    result
}
