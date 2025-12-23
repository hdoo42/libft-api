use std::{collections::HashMap, io::Write, ops::ControlFlow, sync::Arc, time::Duration};

use chrono::Utc;
use libft_api::{campus_id::*, prelude::*, FT_CURSUS_ID, FT_PISCINE_CURSUS_ID};
use rvstruct::ValueStruct;
use tokio::{sync::Semaphore, task::JoinSet, time::sleep};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let thread_num = 8;
    let permit = Arc::new(Semaphore::new(thread_num));

    let ids = [
        // use same users
    ]
    .map(FtUserId::new);
    let mut handles = JoinSet::new();

    for id in ids {
        let permit = Arc::clone(&permit);
        handles.spawn(async move {
            let _permit = permit.acquire().await.unwrap();
            let mut result = HashMap::new();
            let mut page = 1;
            loop {
                if let ControlFlow::Break(()) =
                    get_evaluation_historics(&mut result, &id, &mut page).await
                {
                    break result;
                }
            }
        });
    }

    let mut historics_of_students = Vec::new();
    while let Some(Ok(res)) = handles.join_next().await {
        historics_of_students.extend(res);
        info!("{}", historics_of_students.len());
    }

    let file_path = format!(
        "./evaluation_historics_{}.csv",
        Utc::now().format("%Y-%m-%d_%H-%M-%S")
    );

    let mut file = std::fs::File::create(&file_path).expect("Failed to create output file");

    file.write_all(
        "id, created_at, reason, scale_team_id, sum, total, updated_at, intra_id\n".as_bytes(),
    )?;

    for (intra_id, historics) in historics_of_students {
        for history in historics {
            writeln!(
                file,
                "{},{},{},{},{},{},{},{}",
                history.id,
                history.created_at.0.to_utc(),
                history.reason,
                history
                    .scale_team_id
                    .map(|team| team.value().to_string())
                    .unwrap_or("".to_string()),
                history.sum,
                history.total,
                history.updated_at.0.to_utc(),
                intra_id
            )
            .expect("Failed to write record");
        }
    }

    // let mut handles = JoinSet::new();
    //
    // let mut scale_teams = Vec::new();
    // for mut page in 1..=thread_num {
    //     let permit = Arc::clone(&permit);
    //     handles.spawn(async move {
    //         let _permit = permit.acquire().await.unwrap();
    //         let mut result = Vec::new();
    //         loop {
    //             if let ControlFlow::Break(()) =
    //                 get_scale_teams(&mut result, &mut page, thread_num).await
    //             {
    //                 break result;
    //             }
    //         }
    //     });
    // }
    //
    // while let Some(Ok(res)) = handles.join_next().await {
    //     scale_teams.extend(res);
    //     info!("{}", scale_teams.len());
    // }
    //
    // let file_path = format!(
    //     "./scale_teams_{}.csv",
    //     Utc::now().format("%Y-%m-%d_%H-%M-%S")
    // );
    //
    // let mut file = std::fs::File::create(&file_path).expect("Failed to create output file");
    //
    // file.write_all("project_idㅣscale_team_idㅣcreated_atㅣupdated_atㅣfinal_markㅣbegin_atㅣcorrectorㅣcorrectedsㅣfilled_atㅣtruantㅣteam.userㅣcommentㅣfeedback\n".as_bytes())?;
    //
    // for scale_team in scale_teams {
    //     let corrector = match scale_team.corrector {
    //         FtCorrector::User(ft_user) => {
    //             ft_user.login.map(|login| login.0).unwrap_or("".to_string())
    //         }
    //         FtCorrector::String(s) => s,
    //     };
    //     let correcteds = match scale_team.correcteds {
    //         FtCorrecteds::String(s) => s,
    //         FtCorrecteds::Vec(vec) => vec
    //             .into_iter()
    //             .map(|user| user.login.map(|l| l.0).unwrap_or("".to_string()))
    //             .collect::<Vec<String>>()
    //             .join(","),
    //     };
    //     let begin_at = match scale_team.begin_at {
    //         Some(date) => date.0.to_utc().to_string(),
    //         None => "".to_string(),
    //     };
    //     let filled_at = match scale_team.filled_at {
    //         Some(date) => date.0.to_utc().to_string(),
    //         None => "".to_string(),
    //     };
    //
    //     let truant = match scale_team.truant {
    //         Some(user) => user
    //             .login
    //             .map(|l| l.0.to_string())
    //             .unwrap_or("".to_string()),
    //         None => "".to_string(),
    //     };
    //     let (team_uesr, project_id) = match scale_team.team {
    //         Some(team) => {
    //             let user = team
    //                 .users
    //                 .map(|users| {
    //                     users
    //                         .into_iter()
    //                         .map(|user| {
    //                             user.login
    //                                 .map(|l| l.0.to_string())
    //                                 .unwrap_or("".to_string())
    //                         })
    //                         .collect::<Vec<String>>()
    //                         .join(",")
    //                 })
    //                 .unwrap_or("".to_string());
    //             let project_id = team
    //                 .project_id
    //                 .map(|project_id| project_id.to_string())
    //                 .unwrap_or("".to_string());
    //             (user, project_id)
    //         }
    //         None => ("".to_string(), "".to_string()),
    //     };
    //     let final_mark = match scale_team.final_mark {
    //         Some(final_mark) => final_mark.value().to_string(),
    //         None => "".to_string(),
    //     };
    //     writeln!(
    //         file,
    //         "{}ㅣ{}ㅣ{}ㅣ{}ㅣ{}ㅣ{}ㅣ{}ㅣ{}ㅣ{}ㅣ{}ㅣ{}ㅣ{:?}ㅣ{:?}",
    //         project_id,
    //         scale_team.id,
    //         scale_team.created_at.0.to_utc(),
    //         scale_team.updated_at.0.to_utc(),
    //         final_mark,
    //         begin_at,
    //         corrector,
    //         correcteds,
    //         filled_at,
    //         truant,
    //         team_uesr,
    //         scale_team.comment,
    //         scale_team.feedback
    //     )
    //     .expect("Failed to write record");
    // }

    println!("Output written to: {}", file_path);
    Ok(())
}

async fn get_evaluation_historics(
    result: &mut HashMap<FtUserId, Vec<FtCorrectionPointHistory>>,
    id: &FtUserId,
    page: &mut i32,
) -> ControlFlow<()> {
    let token = FtApiToken::try_get(AuthInfo::build_from_env().unwrap())
        .await
        .unwrap();
    let client = FtClient::new(FtClientReqwestConnector::new());
    let session = Arc::new(client.open_session(&token));
    let res = session
        .users_id_correction_point_historics(
            FtApiUsersIdCorrectionPointHistoricsRequest::new(*id)
                .with_filter(vec![FtFilterOption::new(
                    FtFilterField::Sum,
                    vec!["-1".to_owned()],
                )])
                .with_per_page(100)
                .with_page(*page as u16),
        )
        .await;
    match res {
        Ok(res) => {
            if res.historics.is_empty() {
                return ControlFlow::Break(());
            }
            result.entry(*id).or_default().extend(res.historics);
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

async fn get_scale_teams(
    result: &mut Vec<FtScaleTeam>,
    page: &mut usize,
    thread_num: usize,
) -> ControlFlow<()> {
    let token = FtApiToken::try_get(AuthInfo::build_from_env().unwrap())
        .await
        .unwrap();
    let client = FtClient::new(FtClientReqwestConnector::new());
    let session = Arc::new(client.open_session(&token));
    let res = session
        .scale_teams(
            FtApiScaleTeamsRequest::new()
                .with_range(vec![FtRangeOption::new(
                    FtRangeField::CreatedAt,
                    vec!["2025-4-21".to_string(), "2025-4-25".to_string()],
                )])
                .with_filter(vec![
                    FtFilterOption::new(FtFilterField::CampusId, vec![GYEONGSAN.to_string()]),
                    FtFilterOption::new(FtFilterField::CursusId, vec![FT_CURSUS_ID.to_string()]),
                ])
                .with_per_page(100)
                .with_page(*page as u16),
        )
        .await;
    match res {
        Ok(res) => {
            if res.scale_teams.is_empty() {
                return ControlFlow::Break(());
            }
            result.extend(res.scale_teams);
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
