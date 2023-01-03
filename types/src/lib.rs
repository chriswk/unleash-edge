use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

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
    NoHttpClient,
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
            Self::UnleashApiError => StatusCode::INTERNAL_SERVER_ERROR,
            Self::UnleashJsonError => StatusCode::BAD_REQUEST,
            Self::FailedToAcquireLock => StatusCode::CONFLICT,
            Self::CouldNotBind => StatusCode::INTERNAL_SERVER_ERROR,
            Self::CouldNotParseQuery => StatusCode::BAD_REQUEST,
            Self::InvalidHeaderValue => StatusCode::BAD_REQUEST,
            Self::AuthorizationDenied => StatusCode::FORBIDDEN,
            Self::NoToken => StatusCode::UNAUTHORIZED,
            Self::NoHttpClient => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        match self.status_code() {
            _ => HttpResponse::build(self.status_code()).finish(),
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
