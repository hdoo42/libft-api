use rsb_derive::Builder;
use serde::{Deserialize, Serialize};

use crate::{prelude::*, to_param};

#[derive(Debug, Serialize, Deserialize, Builder)]
pub struct FtApiUsersIdRequest {
    pub id: FtUserIdentifier,
    pub sort: Option<Vec<FtSortOption>>,
    pub range: Option<Vec<FtRangeOption>>,
    pub filter: Option<Vec<FtFilterOption>>,
    pub page: Option<usize>,
    pub per_page: Option<u8>,
}

#[derive(Debug, Serialize, Deserialize, Builder)]
#[serde(transparent)]
pub struct FtApiUsersIdResponse {
    pub user: FtUser,
}

impl<FCHC> FtClientSession<'_, FCHC>
where
    FCHC: FtClientHttpConnector + Send + Sync,
{
    /// Retrieves information about a specific user from the 42 Intra API.
    ///
    /// This method fetches detailed information about a user identified by either their user ID
    /// or login name. The method supports various query parameters for filtering, sorting, and pagination.
    ///
    /// # Parameters
    /// - `req`: A `FtApiUsersIdRequest` struct containing the query parameters, including the user identifier.
    ///
    /// # Query Parameters
    /// - `id`: The identifier for the user (either user ID or login name)
    /// - `sort`: Optional vector of sort options to order the results
    /// - `range`: Optional vector of range options to filter results by date ranges
    /// - `filter`: Optional vector of filter options to filter the results
    /// - `page`: Optional page number for pagination
    /// - `per_page`: Optional number of items per page for pagination
    ///
    /// # Returns
    /// - `ClientResult<FtApiUsersIdResponse>`: Contains a `FtUser` object with detailed user information
    ///
    /// # Example
    /// ```rust
    /// use libft_api::prelude::*;
    ///
    /// async fn example() -> ClientResult<()> {
    ///     let token = FtApiToken::try_get(AuthInfo::build_from_env().unwrap()).await.unwrap();
    ///     let client = FtClient::new(FtClientReqwestConnector::new());
    ///     let session = client.open_session(token);
    ///
    ///     // Get user by ID
    ///     let user_by_id = session
    ///         .users_id(
    ///             FtApiUsersIdRequest::new(FtUserIdentifier::UserId(FtUserId::new(12345)))
    ///         )
    ///         .await?;
    ///     println!("User name: {:?} {:?}", user_by_id.user.first_name, user_by_id.user.last_name);
    ///
    ///     // Get user by login
    ///     let user_by_login = session
    ///         .users_id(
    ///             FtApiUsersIdRequest::new(FtUserIdentifier::Login(
    ///                 FtLoginId::new("user_login".to_string())
    ///             ))
    ///         )
    ///         .await?;
    ///     println!("User login: {:?}", user_by_login.user.login);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn users_id(&self, req: FtApiUsersIdRequest) -> ClientResult<FtApiUsersIdResponse> {
        let url = &format!(
            "users/{}",
            match req.id {
                FtUserIdentifier::Login(ft_login_id) => ft_login_id.to_string(),
                FtUserIdentifier::UserId(ft_user_id) => ft_user_id.to_string(),
            }
        );
        let filters = convert_filter_option_to_tuple(req.filter.unwrap_or_default()).unwrap();
        let range = convert_range_option_to_tuple(req.range.unwrap_or_default()).unwrap();

        let params = vec![
            to_param!(req, page),
            to_param!(req, per_page),
            (
                "sort".to_string(),
                req.sort.as_ref().map(|v| {
                    v.iter()
                        .map(|v| {
                            format!(
                                "{}{}",
                                if v.descending { "-" } else { "" },
                                serde_plain::to_string(&v.field).unwrap()
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(",")
                }),
            ),
        ];

        self.http_session_api
            .http_get(url, &[filters, range, params].concat())
            .await
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn basic() {
        let token = FtApiToken::try_get(AuthInfo::build_from_env().unwrap())
            .await
            .unwrap();

        let client = FtClient::new(FtClientReqwestConnector::with_connector(
            reqwest::Client::new(),
        ));

        let session = client.open_session(token);
        /* let res = */
        session
            .users_id(
                FtApiUsersIdRequest::new(FtUserIdentifier::Login(FtLoginId::new(
                    "taejikim".to_owned(),
                )))
                .with_per_page(1),
            )
            .await
            .unwrap();

        // assert!(res.is_ok());
    }
}
