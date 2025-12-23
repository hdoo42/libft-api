//! API endpoints related to scale team information.
//!
//! This module provides access to the 42 Intra API endpoints that deal with scale team data.
//! Scale teams are evaluation teams used for peer reviews and project assessments.
//! It includes functionality for retrieving scale teams and creating multiple scale teams at once.
//!
//! # Endpoints
//!
//! * **scale_teams**: Retrieve a list of scale teams with filtering, pagination, and sorting options
//! * **scale_teams_multiple_create_post**: Create multiple scale teams at once
//!
//! # Example
//!
//! ```rust
//! use libft_api::prelude::*;
//!
//! async fn example() -> ClientResult<()> {
//!     let token = FtApiToken::try_get(AuthInfo::build_from_env().unwrap()).await.unwrap();
//!     let client = FtClient::new(FtClientReqwestConnector::new());
//!     let session = client.open_session(token);
//!
//!     // Get scale teams with filtering
//!     let response = session
//!         .scale_teams(
//!             FtApiScaleTeamsRequest::new()
//!                 .with_filter(vec![
//!                     FtFilterOption::new(FtFilterField::CampusId, vec!["1".to_owned()]) // Paris campus
//!                 ])
//!         )
//!         .await?;
//!     println!("Found {} scale teams", response.scale_teams.len());
//!
//!     // Create multiple scale teams at once (if you have the appropriate permissions)
//!     // let create_request = FtApiScaleTeamsMultipleCreateRequest::new(vec![
//!     //     FtApiScaleTeamsMultipleCreateBody {
//!     //         begin_at: FtDateTimeUtc::now(),
//!     //         user_id: FtUserId::new(12345),
//!     //         team_id: FtTeamId::new(67890),
//!     //     }
//!     // ]);
//!     // let created_teams = session.scale_teams_multiple_create_post(create_request).await?;
//!
//!     Ok(())
//! }
//! ```

mod scale_teams;
pub use scale_teams::*;
mod scale_teams_id;
pub use scale_teams_id::*;
