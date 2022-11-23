use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub enum EdgeError {
    UnleashApiError,
    UnleashJsonError,
    FailedToAcquireLock,
    CouldNotBind,
}

pub type EdgeResult<T> = Result<T, EdgeError>;

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

impl FromStr for EdgeToken {
    type Err = EdgeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let token_parts: Vec<String> = s
            .clone()
            .split(":")
            .take(2)
            .map(|s| s.to_string())
            .collect();
        let token_projects = if token_parts.get(0).unwrap() == "[]" {
            vec![]
        } else {
            vec![token_parts.get(0).unwrap().clone()]
        };
        let env_token: Vec<String> = token_parts
            .get(1)
            .unwrap()
            .split(".")
            .take(2)
            .map(|s| s.to_string())
            .collect();
        Ok(EdgeToken {
            token: env_token.get(1).unwrap().clone(),
            environment: env_token.get(0).unwrap().clone(),
            projects: token_projects,
            token_type: ApiToken::Undecided,
            dynamic: false,
        })
    }
}
