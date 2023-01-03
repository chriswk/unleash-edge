use crate::{ApiToken, EdgeError, EdgeResult, EdgeToken};
use actix_utils::future::{ready, Ready};
use actix_web::dev::Payload;
use actix_web::http::header::HeaderValue;
use actix_web::{FromRequest, HttpRequest};
use std::str::FromStr;

impl FromRequest for EdgeToken {
    type Error = EdgeError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let value: Option<EdgeResult<EdgeToken>> = req
            .headers()
            .get("Authorization")
            .map(|auth_header| EdgeToken::try_from(auth_header.clone()));
        ready(value.unwrap_or(Err(EdgeError::NoToken)))
    }
}

impl TryFrom<HeaderValue> for EdgeToken {
    type Error = EdgeError;

    fn try_from(value: HeaderValue) -> Result<Self, Self::Error> {
        value
            .to_str()
            .map_err(|_| EdgeError::InvalidHeaderValue)
            .and_then(|t| EdgeToken::from_str(t))
    }
}

impl FromStr for EdgeToken {
    type Err = EdgeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let token_parts: Vec<String> = s
            .clone()
            .split(":")
            .take(2)
            .map(|s| s.to_string())
            .collect();
        let token_projects = if let Some(projects) = token_parts.get(0) {
            if projects == "[]" {
                vec![]
            } else {
                vec![projects.clone()]
            }
        } else {
            return Err(EdgeError::AuthorizationDenied);
        };
        if let Some(env_and_key) = token_parts.get(1) {
            let e_a_k: Vec<String> = env_and_key
                .split(".")
                .take(2)
                .map(|s| s.to_string())
                .collect();
            if e_a_k.len() != 2 {
                return Err(EdgeError::AuthorizationDenied);
            }
            Ok(EdgeToken {
                token: e_a_k.get(1).unwrap().clone(),
                environment: e_a_k.get(0).unwrap().clone(),
                projects: token_projects,
                token_type: ApiToken::Undecided,
                dynamic: false,
            })
        } else {
            return Err(EdgeError::AuthorizationDenied);
        }
    }
}
