use clap::Parser;
use libft_api::prelude::*;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// List of scale_team IDs to patch
    #[arg(short, long, value_parser, num_args = 1.., value_delimiter = ' ')]
    scale_team_ids: Vec<i32>,

    /// The new scale_id to set
    #[arg(short, long)]
    new_scale_id: i32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();

    let token = FtApiToken::try_get(AuthInfo::build_from_env()?)
        .await
        .map_err(|e| format!("Token error: {:?}", e))?;
    let client = FtClient::new(FtClientReqwestConnector::new());
    let session = client.open_session(token);

    println!(
        "Patching {} scale teams to scale_id: {}",
        args.scale_team_ids.len(),
        args.new_scale_id
    );

    for id in args.scale_team_ids {
        let scale_team_id = FtScaleTeamId::new(id);
        let new_scale_id = FtScaleId::new(args.new_scale_id);

        match session
            .scale_teams_id_patch(FtApiScaleTeamsIdPatchRequest::new(
                scale_team_id,
                new_scale_id,
            ))
            .await
        {
            Ok(_) => println!("Successfully patched scale_team {}", id),
            Err(e) => eprintln!("Failed to patch scale_team {}: {}", id, e),
        }
    }

    Ok(())
}
