use std::fmt::{Display, Formatter};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use actix_web::body::BoxBody;

pub mod extractors;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub enum EdgeError {
    UnleashApiError,
    UnleashJsonError,
    FailedToAcquireLock,
    CouldNotBind,
    CouldNotParseQuery,
    InvalidHeaderValue,
    AuthorizationDenied,
    NoToken,
}

impl Display for EdgeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&*self.to_string())
    }
}

pub type EdgeResult<T> = Result<T, EdgeError>;

impl ResponseError for EdgeError {
    fn status_code(&self) -> StatusCode {
        match self {
            UnleashApiError => StatusCode::INTERNAL_SERVER_ERROR,
            UnleashJsonError => StatusCode::BAD_REQUEST,
            FailedToAcquireLock => StatusCode::CONFLICT,
            CouldNotBind => StatusCode::INTERNAL_SERVER_ERROR,
            CouldNotParse => StatusCode::BAD_REQUEST,
            InvalidHeaderValue=> StatusCode::BAD_REQUEST,
            AuthorizationDenied => StatusCode::FORBIDDEN,
            NoToken => StatusCode::UNAUTHORIZED,
        }
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        match self.status_code() {
            _ => HttpResponse::build(self.status_code()).finish()
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub enum ApiToken {
    Undecided,
    Decided(ApiTokenType),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum ApiTokenType {
    Client,
    Admin,
    Frontend,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct EdgeToken {
    pub token: String,
    pub environment: String,
    pub projects: Vec<String>,
    pub token_type: ApiToken,
    pub dynamic: bool,
}


