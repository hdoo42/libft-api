use std::{io::Write, sync::Arc};

use futures::FutureExt;
use libft_api::{info::ft_campus_id::GYEONGSAN, prelude::*};
use tokio::task::JoinSet;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let client = Arc::new(FtClient::new(FtClientReqwestConnector::new()));

    let req: ReqFn<_> = |session, page| {
        async move {
            session
                .users(
                    FtApiUsersRequest::new()
                        .with_page(page)
                        .with_per_page(100)
                        .with_filter(vec![FtFilterOption::new(
                            FtFilterField::PrimaryCampusId,
                            vec![GYEONGSAN.to_string()],
                        )]),
                )
                .await
        }
        .boxed()
    };

    let mut handles = JoinSet::new();
    let mut result = Vec::new();

    let client_clone = Arc::clone(&client);
    handles.spawn(async move { scroller(&client_clone, 8, 1, req).await });
    if let Some(Ok(res)) = handles.join_next().await {
        result.extend(res);
    }

    for i in 2..=8 {
        let client = Arc::clone(&client);
        handles.spawn(async move { scroller(&client, 8, i, req).await });
    }

    while let Some(res) = handles.join_next().await {
        match res {
            Ok(v) => result.extend(v),
            Err(e) => tracing::error!("task failed: {e}"),
        }
    }

    let mut file = std::fs::File::create("whole.json").unwrap();
    file.write_all(serde_json::to_string_pretty(&result).unwrap().as_bytes())
        .unwrap();
}
