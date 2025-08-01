use std::{io::Write, ops::ControlFlow, sync::Arc, time::Duration};

use chrono::Utc;
use libft_api::{campus_id::GYEONGSAN, prelude::*};
use rvstruct::ValueStruct;
use tokio::{sync::Semaphore, task::JoinSet, time::sleep};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let thread_num = 4;
    let permit = Arc::new(Semaphore::new(thread_num));
    // let ids = [
    //     212531, 212530, 212529, 212528, 212527, 212526, 212525, 212524, 212523, 212522, 212521,
    //     212520, 212519, 212518, 212517, 212516, 212515, 212514, 212513, 212512, 212511, 212510,
    //     212509, 212508, 212507, 212506, 212505, 212504, 212503, 212502, 212501, 212500, 212499,
    //     212498, 212497, 212496, 212495, 212494, 212493, 212492, 212491, 212490, 212489, 212488,
    //     212487, 212486, 212485, 212484, 212483, 212482, 212481, 212480, 212479, 212478, 212477,
    //     212476, 212475, 212474, 212473, 212472, 212471, 212470, 212469, 212468, 212467, 212466,
    //     212465, 212464, 212463, 212462, 212461, 212460, 212459, 212458, 212457, 212456, 212455,
    //     212454, 212453, 212452, 212638, 212637, 212629, 212628, 212627, 212626, 212625, 212624,
    //     212623, 212622, 212621, 212620, 212619, 212618, 212617, 212616, 212615, 212614, 212613,
    //     212612, 212611, 212610, 212609, 212608, 212607, 212606, 212605, 212604, 212603, 212602,
    //     212601, 212600, 212599, 212598, 212597, 212596, 212595, 212594, 212593, 212592, 212591,
    //     212590, 212589, 212588, 212587, 212586, 212585, 212584, 212583, 212582, 212581, 212580,
    //     212579, 212578, 212577, 212576, 212575, 212574, 212573, 212572, 212571, 212570, 212569,
    //     212568, 212567, 212566, 212565, 212564, 212563, 212562, 212561, 212560, 212559, 212558,
    //     212557, 212556, 212555, 212554, 212553, 212552, 212551, 212550, 212549, 212548, 212547,
    //     212546, 212545, 212544, 212543, 212542, 212541, 212540, 212539, 212538, 212537, 212536,
    //     212535, 212534, 212533, 212532,
    // ]
    // .map(FtUserId::new);

    let mut handles = JoinSet::new();

    for mut page in 1..=thread_num {
        let permit = Arc::clone(&permit);
        handles.spawn(async move {
            let _permit = permit.acquire().await.unwrap();
            let mut result = Vec::new();
            loop {
                if let ControlFlow::Break(()) = get_users(&mut result, thread_num, &mut page).await
                {
                    break result
                        .into_iter()
                        .filter_map(|user| user.id)
                        .collect::<Vec<FtUserId>>();
                }
            }
        });
    }

    let mut ids = Vec::new();
    while let Some(Ok(res)) = handles.join_next().await {
        ids.extend(res);
    }
    info!("{:#?}", ids);

    let mut handles = JoinSet::new();

    let mut result = Vec::new();
    for id in ids {
        let permit = Arc::clone(&permit);
        handles.spawn(async move {
            let _permit = permit.acquire().await.unwrap();
            let mut result = Vec::new();
            let mut page = 1;
            loop {
                if let ControlFlow::Break(()) =
                    get_projects_users(&mut result, &id, &mut page).await
                {
                    break result;
                }
            }
        });
    }

    while let Some(Ok(res)) = handles.join_next().await {
        result.extend(res);
        info!("{}", result.len());
    }

    let file_path = format!(
        "/Users/hdoo/works/gsia/codes/libft-api/libft-api/bin/piscine/third_cohort/first_round/progress_{}.csv",
        Utc::now().format("%Y-%m-%d_%H-%M-%S")
    );

    let mut file = std::fs::File::create(&file_path).expect("Failed to create output file");

    file.write_all(
        "user_id,login,project_name,marked_at,created_at,final_mark,updated_at\n".as_bytes(),
    )?;

    for projects_user in result {
        let (id, login) = {
            let user = projects_user
                .user
                .expect("projects_users always have user.");
            (
                user.id.map(|id| id.to_string()).unwrap_or("".to_string()),
                user.login
                    .map(|id| id.to_string())
                    .unwrap_or("".to_string()),
            )
        };
        writeln!(
            file,
            "{},{},{},{:?},{},{:?},{}",
            id,
            login,
            projects_user.project.name,
            projects_user.marked_at,
            projects_user.created_at.value(),
            projects_user.final_mark,
            Utc::now()
        )
        .expect("Failed to write record");
    }

    println!("Output written to: {}", file_path);
    Ok(())
}

async fn get_projects_users(
    result: &mut Vec<FtProjectsUser>,
    id: &FtUserId,
    page: &mut i32,
) -> ControlFlow<()> {
    let token = FtApiToken::try_get(AuthInfo::build_from_env().unwrap())
        .await
        .unwrap();
    let client = FtClient::new(FtClientReqwestConnector::new());
    let session = Arc::new(client.open_session(&token));
    let res = session
        .users_id_projects_users(
            FtApiUsersIdProjectsUsersRequest::new(*id)
                .with_per_page(100)
                .with_page(*page as u16),
        )
        .await;
    match res {
        Ok(res) => {
            if res.projects_users.is_empty() {
                return ControlFlow::Break(());
            }
            result.extend(res.projects_users);
            *page += 1;
        }
        Err(FtClientError::RateLimitError(_)) => sleep(Duration::new(1, 42)).await,
        Err(e) => {
            eprintln!("other error: {e}");
            return ControlFlow::Break(());
        }
    }
    ControlFlow::Continue(())
}

async fn get_users(
    result: &mut Vec<FtUser>,
    thread_num: usize,
    page: &mut usize,
) -> ControlFlow<()> {
    let token = FtApiToken::try_get(AuthInfo::build_from_env().unwrap())
        .await
        .unwrap();
    let client = FtClient::new(FtClientReqwestConnector::new());
    let session = Arc::new(client.open_session(&token));
    let res = session
        .users(
            FtApiUsersRequest::new()
                .with_per_page(100)
                .with_page(*page as u16)
                .with_range(vec![FtRangeOption::new(
                    FtRangeField::CreatedAt,
                    vec!["2025-1-1".to_string(), "2025-2-1".to_string()],
                )])
                .with_filter(vec![
                    FtFilterOption::new(
                        FtFilterField::PrimaryCampusId,
                        vec![GYEONGSAN.to_string()],
                    ),
                    FtFilterOption::new(FtFilterField::Kind, vec!["student".to_string()]),
                ]),
        )
        .await;

    match res {
        Ok(res) => {
            if res.users.is_empty() {
                return ControlFlow::Break(());
            }
            res.users
                .iter()
                .for_each(|user| println!("{:?}, {:?}", user.id, user.login));
            result.extend(res.users);
            info!("{}", result.len());
            *page += thread_num;
        }
        Err(FtClientError::RateLimitError(_)) => sleep(Duration::new(1, 42)).await,
        Err(e) => {
            eprintln!("other error: {e}");
            return ControlFlow::Break(());
        }
    }
    ControlFlow::Continue(())
}
