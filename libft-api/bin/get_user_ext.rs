use std::{collections::HashMap, io::Write, ops::ControlFlow, sync::Arc, time::Duration};

use chrono::Utc;
use libft_api::{campus_id::*, prelude::*, FT_PISCINE_CURSUS_ID};
use rvstruct::ValueStruct;
use tokio::{sync::Semaphore, task::JoinSet, time::sleep};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let thread_num = 8;
    let permit = Arc::new(Semaphore::new(thread_num));

    let ids = [
        172410, 197482, 190887, 172305, 172353, 197422, 197429, 190783, 197456, 190848, 190815,
        172394, 174189, 190846, 174084, 190820, 172357, 190800, 197497, 172418, 172352, 172349,
        197528, 190909, 174169, 197496, 174101, 197397, 174128, 174104, 174127, 174112, 197454,
        174184, 197455, 197495, 197484, 172327, 197507, 190797, 197498, 197444, 174097, 190898,
        172325, 174113, 172307, 174153, 172346, 172356, 190862, 197402, 174156, 190839, 197518,
        197483, 174185, 174152, 174145, 197459, 197504, 174131, 190847, 197523, 197521, 197511,
        197406, 197403, 172364, 197486, 172362, 190795, 190802, 197525, 174188, 197457, 190806,
        174089, 174135, 174129, 197400, 190817, 174081, 174147, 197489, 172308, 197463, 190913,
        197437, 197605, 172400, 197516, 190885, 197449, 174161, 174186, 174110, 197439, 190838,
        172329, 190870, 172370, 174085, 174111, 190849, 172416, 190876, 197606, 197519, 174138,
        174149, 172413, 190845, 197527, 190895, 174168, 174137, 172414, 190832, 197537, 172375,
        197441, 174151, 190808, 197472, 172390, 197520, 190843, 172348, 172392, 190896, 172389,
        197448, 197417, 174139, 190907, 172335, 174095, 197494, 190910, 190816, 197445, 197541,
        174130, 174150, 190823, 197467, 190821, 190784, 190926, 174142, 197421, 197420, 174093,
        197435, 197453, 197530, 174102, 190886, 190861, 174103, 197447, 174123, 174099, 174096,
        174178, 172350, 197543, 197474, 174117, 172402, 172324, 172367, 190790, 197490, 190803,
        174133, 197529, 190855, 197428, 197542, 197499, 190837, 190865, 174154, 197547, 197501,
        190812, 190818, 197418, 172310, 190836, 197540, 172342, 190869, 197407, 197533, 190911,
        197487, 172318, 190903, 190831, 190937, 174109, 174115, 190854, 190866, 174181, 190813,
        174091, 172361, 172344, 190785, 197505, 197532, 197531, 172309, 172323, 174157, 197514,
        190791, 174105, 190810, 174183, 190794, 197395, 197458, 197481, 190905, 197412, 174086,
        197548, 197536, 172351, 190829, 174165, 197503, 172385, 172404, 197526, 172365, 197399,
        197538, 172401, 197409, 174119, 174083, 174177, 197539, 197432, 190874, 190844, 172319,
        174141, 190786, 174087, 172378, 190883, 172396, 174160, 190884, 174092, 174132, 197442,
        197398, 174190, 190853, 172330, 197413, 197469, 174094, 172366, 172368, 172322, 197427,
        174120, 197408, 197425, 172360, 197434, 172399, 173488, 151095, 212592, 212527, 212590,
        212600, 212458, 212489, 212601, 212464, 212628, 212493, 212582, 212591, 212469, 212456,
        212608, 212615, 212498, 212625, 212562, 212512, 212612, 212468, 212571, 212471, 212606,
        212560, 212525, 212501, 212572, 212587, 212452, 212460, 212496, 212557, 212476, 212529,
        212534, 212586, 212543, 212602, 212567, 212524, 212477, 212481, 212561, 212473, 212495,
        212522, 212570, 212517, 212538, 212539, 212459, 212462, 212544, 212482, 212558, 212559,
        212457, 212472, 212548, 212553, 212609, 212583, 212535, 212518, 212467, 212521, 212545,
        212533, 212568, 212595, 212505, 212465, 212503, 212499, 212514, 212624, 212466, 212454,
        212549, 212540, 212487, 212555, 212497, 212556, 212623, 212494, 212530, 212581, 212502,
        212510, 212546, 212579,
    ]
    .map(FtUserId::new);

    let mut users_task = JoinSet::new();
    for id in ids {
        let permit = Arc::clone(&permit);
        users_task.spawn(async move {
            let _permit = permit.acquire().await.unwrap();
            loop {
                if let ControlFlow::Break(result) = get_user_info(id).await {
                    break result;
                }
            }
        });
    }

    let file_path = format!(
        "/Users/hdoo/works/gsia/codes/libft-api/libft-api/bin/ft_cursus/info_{}.csv",
        Utc::now().format("%Y-%m-%d_%H-%M-%S")
    );

    let mut file = std::fs::File::create(&file_path).expect("Failed to create output file");

    file.write_all("user_id|login|level\n".as_bytes())?;

    let mut users = Vec::new();
    while let Some(Ok(Some(user))) = users_task.join_next().await {
        users.push(user);
    }

    std::fs::write(
        "/Users/hdoo/works/gsia/codes/libft-api/libft-api/bin/ft_cursus/info_{}.csv",
        serde_json::to_string_pretty(&users)?,
    )?;

    println!("Output written to: {}", file_path);
    Ok(())
}

async fn get_user_info(id: FtUserId) -> ControlFlow<Option<FtUserExt>> {
    let token = FtApiToken::try_get(AuthInfo::build_from_env().unwrap())
        .await
        .unwrap();
    let client = FtClient::new(FtClientReqwestConnector::new());
    let session = Arc::new(client.open_session(&token));
    let res = session
        .users_id(FtApiUsersIdRequest::new(FtUserIdentifier::UserId(id)))
        .await;

    match res {
        Ok(res) => ControlFlow::Break(Some(res.user)),
        Err(e) => match e {
            FtClientError::RateLimitError(ft_rate_limit_error) => {
                sleep(Duration::new(1, 42)).await;
                ControlFlow::Continue(())
            }
            _ => {
                tracing::error!("{:?}", e);
                ControlFlow::Break(None)
            }
        },
    }
}
