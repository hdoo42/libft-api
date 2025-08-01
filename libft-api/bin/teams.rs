use std::{io::Write, sync::Arc};

use chrono::{TimeDelta, TimeZone, Utc};
use ft_project_session_ids::c_piscine::C_PISCINE_RUSH_02;
use libft_api::{campus_id::*, prelude::*, FT_PISCINE_CURSUS_ID};
use rvstruct::ValueStruct;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let begin_at = Utc.with_ymd_and_hms(2025, 1, 28, 7, 0, 0).unwrap();
    let body = vec![
        FtApiScaleTeamsMultipleCreateBody {
            begin_at: FtDateTimeUtc::new(begin_at.clone()),
            user_id: FtUserId::new(174094),
            team_id: FtTeamId::new(6298862),
        },
        FtApiScaleTeamsMultipleCreateBody {
            begin_at: FtDateTimeUtc::new(begin_at),
            user_id: FtUserId::new(172309),
            team_id: FtTeamId::new(6298846),
        },
    ];
    let res = post_scale_team(body).await.unwrap();
    println!("{res:?}");

    Ok(())
}

async fn temp() {
    tracing_subscriber::fmt::init();

    let evaluators = [174094, 172309].map(FtUserId::new);

    let project_teams = get_project_teams(
        FtProjectSessionId::new(C_PISCINE_RUSH_02),
        "2025-1-20".to_string(),
        "2025-2-15".to_string(),
    )
    .await
    .teams;

    project_teams
        .iter()
        .for_each(|teams| println!("{}|{:?}", teams.id, teams.users));

    let begin_at = Utc.with_ymd_and_hms(2025, 1, 28, 5, 0, 0).unwrap();
    let mut bodys = Vec::new();
    for (i, project_team) in project_teams.iter().enumerate() {
        let evaluator = evaluators.get(i % evaluators.len()).unwrap().clone();
        let iter = i / evaluators.len();
        let begin_at = begin_at
            .checked_add_signed(TimeDelta::new(iter as i64 * 60 * 60 * 1, 0).unwrap())
            .map(FtDateTimeUtc::new)
            .unwrap();
        bodys.push(FtApiScaleTeamsMultipleCreateBody {
            begin_at,
            user_id: evaluator,
            team_id: project_team.id.clone(),
        });
    }

    for ele in bodys.iter() {
        println!("{},{},{}", ele.user_id, ele.team_id, ele.begin_at.value());
    }
}

async fn post_scale_team(
    bodys: Vec<FtApiScaleTeamsMultipleCreateBody>,
) -> Result<FtApiScaleTeamsMultipleCreateResponse, libft_api::FtClientError> {
    let token = FtApiToken::try_get(AuthInfo::build_from_env().unwrap())
        .await
        .unwrap();
    let client = FtClient::new(FtClientReqwestConnector::new());
    let session = Arc::new(client.open_session(&token));

    session
        .scale_teams_multiple_create_post(FtApiScaleTeamsMultipleCreateRequest::new(bodys))
        .await
}

async fn get_project_teams(
    project_session_id: FtProjectSessionId,
    begin_at: String,
    end_at: String,
) -> FtApiProjectSessionsTeamsResponse {
    let token = FtApiToken::try_get(AuthInfo::build_from_env().unwrap())
        .await
        .unwrap();
    let client = FtClient::new(FtClientReqwestConnector::new());
    let session = Arc::new(client.open_session(&token));
    let res = session
        .project_sessions_id_teams(
            FtApiProjectSessionsTeamsRequest::new(project_session_id)
                .with_per_page(100)
                .with_filter(vec![
                    FtFilterOption::new(FtFilterField::Campus, vec![GYEONGSAN.to_string()]),
                    FtFilterOption::new(
                        FtFilterField::Cursus,
                        vec![FT_PISCINE_CURSUS_ID.to_string()],
                    ),
                ])
                .with_range(vec![FtRangeOption::new(
                    FtRangeField::CreatedAt,
                    vec![begin_at, end_at],
                )]),
        )
        .await;

    res.unwrap()
}
