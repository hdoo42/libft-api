use crate::prelude::*;
use rsb_derive::Builder;
use rvstruct::ValueStruct;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Builder)]
pub struct FtApiScaleTeamsIdRequest {
    pub id: FtScaleTeamId,
}

#[derive(Debug, Serialize, Deserialize, Builder)]
#[serde(transparent)]
pub struct FtApiScaleTeamsIdResponse {
    pub scale_teams: FtScaleTeam,
}

#[derive(Debug, Serialize, Deserialize, Builder)]
pub struct FtApiScaleTeamsIdPatchRequest {
    pub id: FtScaleTeamId,
    pub scale_id: FtScaleId,
}

pub struct FtApiScaleTeamsIdPatchBody {}

#[derive(Debug, Serialize, Deserialize)]
pub struct FtApiScaleTeamsIdPatchResponse {}

impl<FCHC> FtClientSession<'_, FCHC>
where
    FCHC: FtClientHttpConnector + Send + Sync,
{
    pub async fn scale_teams_id_patch(
        &self,
        req: FtApiScaleTeamsIdPatchRequest,
    ) -> ClientResult<FtApiScaleTeamsIdPatchResponse> {
        let url = &format!(
            "scale_teams/{}?scale_team[scale_id]={}",
            req.id.value(),
            req.scale_id.value()
        );
        let body = serde_json::json!({});
        self.http_session_api.http_patch(url, &body).await
    }

    pub async fn scale_teams_id(
        &self,
        req: FtApiScaleTeamsIdRequest,
    ) -> ClientResult<FtApiScaleTeamsIdResponse> {
        let url = &format!("scale_teams/{}", req.id.value());
        self.http_session_api
            .http_get(url, &FT_HTTP_EMPTY_GET_PARAMS.clone())
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_switch_scale_id() {
        let token = FtApiToken::try_get(AuthInfo::build_from_env().unwrap())
            .await
            .unwrap();
        let client = FtClient::new(FtClientReqwestConnector::new());
        let session = client.open_session(token);

        let scale_team_id = FtScaleTeamId::new(8980892);
        let old_scale_id = FtScaleId::new(45833);
        let new_scale_id = FtScaleId::new(55193);

        // Switch to new
        let res = session
            .scale_teams_id_patch(FtApiScaleTeamsIdPatchRequest::new(
                scale_team_id.clone(),
                new_scale_id.clone(),
            ))
            .await;
        assert!(res.is_ok(), "Failed to switch to new scale_id: {:?}", res);
        let res = session
            .scale_teams_id(FtApiScaleTeamsIdRequest::new(FtScaleTeamId::new(8980892)))
            .await;
        assert_eq!(res.unwrap().scale_teams.scale_id, new_scale_id);

        // Switch back to old
        let res = session
            .scale_teams_id_patch(FtApiScaleTeamsIdPatchRequest::new(
                scale_team_id,
                old_scale_id.clone(),
            ))
            .await;
        assert!(
            res.is_ok(),
            "Failed to switch back to old scale_id: {:?}",
            res
        );
        let res = session
            .scale_teams_id(FtApiScaleTeamsIdRequest::new(FtScaleTeamId::new(8980892)))
            .await;
        assert_eq!(res.unwrap().scale_teams.scale_id, old_scale_id);
    }

    #[tokio::test]
    async fn scale_teams_id() {
        let token = FtApiToken::try_get(AuthInfo::build_from_env().unwrap())
            .await
            .unwrap();
        let client = FtClient::new(FtClientReqwestConnector::new());
        let session = client.open_session(token);

        let res = session
            .scale_teams_id(FtApiScaleTeamsIdRequest::new(FtScaleTeamId::new(8980892)))
            .await;

        assert!(res.is_ok());
    }
}
