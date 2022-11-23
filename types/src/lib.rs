use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub enum EdgeError {
    UnleashApiError,
    UnleashJsonError,
}

pub type EdgeResult<T> = Result<T, EdgeError>;
