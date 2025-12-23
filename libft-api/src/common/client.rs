use futures::{future::BoxFuture, FutureExt};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use url::Url;

use crate::auth::FtApiToken;
use crate::common::*;
use crate::connector::*;

/// Type alias for client operation results.
///
/// This is a convenience type alias that represents the result of API operations,
/// returning either a success value of type T or an error of type FtClientError.
pub type ClientResult<T> = std::result::Result<T, FtClientError>;

/// Type alias for the default reqwest-based client implementation.
///
/// This is a convenience type alias that represents an FtClient configured with the
/// FtClientReqwestConnector, which uses the reqwest HTTP client library.
pub type FtReqwestClient = FtClient<FtClientReqwestConnector>;

/// The main client for interacting with the 42 Intra API.
///
/// The FtClient is the primary entry point for making API requests to the 42 Intra API.
/// It manages the HTTP connector, rate limiting, and provides methods to open sessions
/// for making authenticated API calls.
///
/// # Example
/// ```rust
/// use libft_api::prelude::*;
///
/// async fn example() -> ClientResult<()> {
///     let client = FtClient::new(FtClientReqwestConnector::new());
///     let token = FtApiToken::try_get(AuthInfo::build_from_env()?).await?;
///     let session = client.open_session(token);
///     
///     // Use the session to make API calls
///     let users = session.users(FtApiUsersRequest::new()).await?;
///     println!("Found {} users", users.users.len());
///     
///     Ok(())
/// }
/// ```
#[derive(Clone, Debug)]
pub struct FtClient<FCHC>
where
    FCHC: FtClientHttpConnector + Send,
{
    pub http_api: FtClientHttpApi<FCHC>,
    pub meta: HeaderMetaData,
}

/// The HTTP API client.
///
/// This structure wraps the HTTP connector and provides the core functionality
/// for making HTTP requests to the 42 Intra API. It is contained within the FtClient
/// and is responsible for managing the underlying HTTP connection.
#[derive(Clone, Debug)]
pub struct FtClientHttpApi<FCHC>
where
    FCHC: FtClientHttpConnector + Send,
{
    /// The HTTP connector.
    pub connector: Arc<FCHC>,
}

/// URI utilities for the 42 API.
///
/// This structure provides static methods for constructing URLs and handling
/// API endpoints, ensuring consistent URL formatting for all API requests.
pub struct FtClientHttpApiUri;

/// A session for making authenticated API requests.
///
/// An FtClientSession represents an authenticated session with a valid API token.
/// It provides methods for making API calls that require authentication.
///
/// The session is created by calling `FtClient::open_session` and holds a reference
/// to the parent client and the authentication token.
#[derive(Debug)]
pub struct FtClientSession<'a, FCHC>
where
    FCHC: FtClientHttpConnector + Send,
{
    pub http_session_api: FtClientHttpSessionApi<'a, FCHC>,
}

/// The HTTP session API for authenticated requests.
///
/// This structure provides the underlying HTTP functionality for authenticated
/// API requests. It holds the authentication token and a reference to the parent
/// client, allowing for authenticated API calls.
#[derive(Debug)]
pub struct FtClientHttpSessionApi<'a, FCHC>
where
    FCHC: FtClientHttpConnector + Send,
{
    token: FtApiToken,
    pub client: &'a FtClient<FCHC>,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct FtEnvelopeMessage {
    pub ok: bool,
    pub error: Option<String>,
    pub errors: Option<Vec<String>>,
    pub warnings: Option<Vec<String>>,
}

/// A trait for an HTTP client that can connect to the 42 API.
pub trait FtClientHttpConnector {
    /// Send an HTTP GET request to the given URI.
    fn http_get_uri<'a, RS>(
        &'a self,
        full_uri: Url,
        token: &'a FtApiToken,
        ratelimiter: &'a HeaderMetaData,
    ) -> BoxFuture<'a, ClientResult<RS>>
    where
        RS: for<'de> serde::de::Deserialize<'de> + Send + 'a;

    /// Send an HTTP GET request to the given relative URI.
    fn http_get<'a, 'p, RS, PT, TS>(
        &'a self,
        method_relative_uri: &str,
        token: &'a FtApiToken,
        ratelimiter: &'a HeaderMetaData,
        params: &'p PT,
    ) -> BoxFuture<'a, ClientResult<RS>>
    where
        RS: for<'de> serde::de::Deserialize<'de> + Send + 'a,
        PT: std::iter::IntoIterator<Item = (String, Option<TS>)> + Clone,
        TS: AsRef<str> + 'p + Send,
    {
        let full_uri = self
            .create_method_uri_path(method_relative_uri)
            .and_then(|url| FtClientHttpApiUri::create_url_with_params(url, params));

        match full_uri {
            Ok(full_uri) => self.http_get_uri(full_uri, token, ratelimiter),
            Err(err) => std::future::ready(Err(err)).boxed(),
        }
    }

    /// Send an HTTP POST request to the given URI.
    fn http_post_uri<'a, RQ, RS>(
        &'a self,
        full_uri: Url,
        token: &'a FtApiToken,
        request_body: &'a RQ,
    ) -> BoxFuture<'a, ClientResult<RS>>
    where
        RQ: serde::ser::Serialize + Send + Sync,
        RS: for<'de> serde::de::Deserialize<'de> + Send + 'a;

    /// Send an HTTP POST request to the given relative URI.
    fn http_post<'a, RQ, RS>(
        &'a self,
        method_relative_uri: &str,
        token: &'a FtApiToken,
        request: &'a RQ,
    ) -> BoxFuture<'a, ClientResult<RS>>
    where
        RQ: serde::ser::Serialize + Send + Sync,
        RS: for<'de> serde::de::Deserialize<'de> + Send + 'a,
    {
        match self.create_method_uri_path(method_relative_uri) {
            Ok(full_uri) => self.http_post_uri(full_uri, token, request),
            Err(err) => std::future::ready(Err(err)).boxed(),
        }
    }

    /// Send an HTTP PATCH request to the given URI.
    fn http_patch_uri<'a, RQ, RS>(
        &'a self,
        full_uri: Url,
        token: &'a FtApiToken,
        request_body: &'a RQ,
    ) -> BoxFuture<'a, ClientResult<RS>>
    where
        RQ: serde::ser::Serialize + Send + Sync,
        RS: for<'de> serde::de::Deserialize<'de> + Send + 'a;

    /// Send an HTTP PATCH request to the given relative URI.
    fn http_patch<'a, RQ, RS>(
        &'a self,
        method_relative_uri: &str,
        token: &'a FtApiToken,
        request: &'a RQ,
    ) -> BoxFuture<'a, ClientResult<RS>>
    where
        RQ: serde::ser::Serialize + Send + Sync,
        RS: for<'de> serde::de::Deserialize<'de> + Send + 'a,
    {
        match self.create_method_uri_path(method_relative_uri) {
            Ok(full_uri) => self.http_patch_uri(full_uri, token, request),
            Err(err) => std::future::ready(Err(err)).boxed(),
        }
    }

    /// Send an HTTP DELETE request to the given URI.
    fn http_delete_uri<'a, RQ, RS>(
        &'a self,
        full_uri: Url,
        token: &'a FtApiToken,
        request_body: &'a RQ,
    ) -> BoxFuture<'a, ClientResult<RS>>
    where
        RQ: serde::ser::Serialize + Send + Sync,
        RS: for<'de> serde::de::Deserialize<'de> + Send + 'a;

    /// Send an HTTP DELETE request to the given relative URI.
    fn http_delete<'a, RQ, RS>(
        &'a self,
        method_relative_uri: &str,
        token: &'a FtApiToken,
        request: &'a RQ,
    ) -> BoxFuture<'a, ClientResult<RS>>
    where
        RQ: serde::ser::Serialize + Send + Sync,
        RS: for<'de> serde::de::Deserialize<'de> + Send + 'a,
    {
        match self.create_method_uri_path(method_relative_uri) {
            Ok(full_uri) => self.http_delete_uri(full_uri, token, request),
            Err(err) => std::future::ready(Err(err)).boxed(),
        }
    }

    /// Create a new `Url` from a relative URI.
    fn create_method_uri_path(&self, method_relative_uri: &str) -> ClientResult<Url> {
        Ok(FtClientHttpApiUri::create_method_uri_path(method_relative_uri).parse()?)
    }
}

impl<FCHC> FtClient<FCHC>
where
    FCHC: FtClientHttpConnector + Send + Sync,
{
    /// Create a new `FtClient` with the given HTTP connector.
    pub fn new(http_connector: FCHC) -> Self {
        Self {
            http_api: FtClientHttpApi::new(Arc::new(http_connector)),
            meta: HeaderMetaData::new(RateLimiter::new(2, 1200)),
        }
    }

    /// Create a new `FtClient` with the given HTTP connector and rate limits.
    pub fn with_ratelimits(http_connector: FCHC, secondly: u64, hourly: u64) -> Self {
        Self {
            http_api: FtClientHttpApi::new(Arc::new(http_connector)),
            meta: HeaderMetaData::new(RateLimiter::new(secondly, hourly)),
        }
    }

    /// Open a new session for the client.
    pub fn open_session(&'_ self, token: FtApiToken) -> FtClientSession<'_, FCHC> {
        // TODO: Add tracer for LOGGING
        // let http_session_span = span!(Level::DEBUG, "Ft API request",);

        let http_session_api = FtClientHttpSessionApi {
            client: self,
            token,
        };

        FtClientSession { http_session_api }
    }
}

impl<FCHC> FtClientHttpApi<FCHC>
where
    FCHC: FtClientHttpConnector + Send,
{
    pub fn new(http_connector: Arc<FCHC>) -> Self {
        Self {
            connector: http_connector,
        }
    }
}

impl<FCHC> FtClientHttpSessionApi<'_, FCHC>
where
    FCHC: FtClientHttpConnector + Send + Sync,
{
    pub async fn http_get_uri<RS, PT, TS>(&self, full_uri: Url) -> ClientResult<RS>
    where
        RS: for<'de> serde::de::Deserialize<'de> + Send,
    {
        self.client
            .http_api
            .connector
            .http_get_uri(full_uri, &self.token, &self.client.meta)
            .await
    }

    pub async fn http_get<'p, RS, PT, TS>(
        &self,
        method_relative_uri: &str,
        params: &'p PT,
    ) -> ClientResult<RS>
    where
        RS: for<'de> serde::de::Deserialize<'de> + Send,
        PT: std::iter::IntoIterator<Item = (String, Option<TS>)> + Clone,
        TS: AsRef<str> + 'p + Send,
    {
        self.client
            .http_api
            .connector
            .http_get(method_relative_uri, &self.token, &self.client.meta, params)
            .await
    }

    pub async fn http_post<RQ, RS>(
        &self,
        method_relative_uri: &str,
        request: &RQ,
    ) -> ClientResult<RS>
    where
        RQ: serde::ser::Serialize + Send + Sync,
        RS: for<'de> serde::de::Deserialize<'de> + Send,
    {
        self.client
            .http_api
            .connector
            .http_post(method_relative_uri, &self.token, request)
            .await
    }

    pub async fn http_post_uri<RQ, RS>(&self, full_uri: Url, request: &RQ) -> ClientResult<RS>
    where
        RQ: serde::ser::Serialize + Send + Sync,
        RS: for<'de> serde::de::Deserialize<'de> + Send,
    {
        self.client
            .http_api
            .connector
            .http_post_uri(full_uri, &self.token, request)
            .await
    }

    pub async fn http_delete<RQ, RS>(
        &self,
        method_relative_uri: &str,
        request: &RQ,
    ) -> ClientResult<RS>
    where
        RQ: serde::ser::Serialize + Send + Sync,
        RS: for<'de> serde::de::Deserialize<'de> + Send,
    {
        self.client
            .http_api
            .connector
            .http_delete(method_relative_uri, &self.token, request)
            .await
    }

    pub async fn http_delete_uri<RQ, RS>(&self, full_uri: Url, request: &RQ) -> ClientResult<RS>
    where
        RQ: serde::ser::Serialize + Send + Sync,
        RS: for<'de> serde::de::Deserialize<'de> + Send,
    {
        self.client
            .http_api
            .connector
            .http_delete_uri(full_uri, &self.token, request)
            .await
    }

    pub async fn http_patch<RQ, RS>(
        &self,
        method_relative_uri: &str,
        request: &RQ,
    ) -> ClientResult<RS>
    where
        RQ: serde::ser::Serialize + Send + Sync,
        RS: for<'de> serde::de::Deserialize<'de> + Send,
    {
        self.client
            .http_api
            .connector
            .http_patch(method_relative_uri, &self.token, request)
            .await
    }

    pub async fn http_patch_uri<RQ, RS>(&self, full_uri: Url, request: &RQ) -> ClientResult<RS>
    where
        RQ: serde::ser::Serialize + Send + Sync,
        RS: for<'de> serde::de::Deserialize<'de> + Send,
    {
        self.client
            .http_api
            .connector
            .http_patch_uri(full_uri, &self.token, request)
            .await
    }
}

lazy_static! {
    pub static ref FT_HTTP_EMPTY_GET_PARAMS: Vec<(String, Option<&'static String>)> = vec![];
    pub static ref FT_HTTP_PAGE_SIZE_100: Vec<(String, Option<&'static str>)> =
        vec![("page[size]".to_string(), Some("100"))];
}

impl FtClientHttpApiUri {
    pub const FT_API_URI_STR: &'static str = "https://api.intra.42.fr/v2";

    pub fn create_method_uri_path(method_relative_uri: &str) -> String {
        format!("{}/{}", Self::FT_API_URI_STR, method_relative_uri)
    }

    pub fn create_url_with_params<'p, PT, TS>(base_url: Url, params: &'p PT) -> ClientResult<Url>
    where
        PT: std::iter::IntoIterator<Item = (String, Option<TS>)> + Clone,
        TS: AsRef<str> + 'p,
    {
        let url_query_params: Vec<(String, String)> = params
            .clone()
            .into_iter()
            .filter_map(|(k, vo)| vo.map(|v| (k, v.as_ref().to_string())))
            .collect();

        Ok(Url::parse_with_params(base_url.as_str(), url_query_params)?)
    }
}
