#[async_trait]
pub trait TokenStore {
    async fn is_valid(token: String) -> EdgeResult<bool>;
    async fn get_tokens() -> EdgeResult<Vec<EdgeToken>>;
    async fn get_token(token: String) -> EdgeResult<Option<IEdgeToken>>;
    async fn add_token(token: EdgeToken) -> EdgeResult<()>;
    async fn remove_token(token: String) -> EdgeResult<()>;
}

#[async_trait]
pub trait ToggleSink {
    async fn save_toggles(env: String, data: UnleashResponse) -> EdgeResult<()>;
}

#[async_trait]
pub trait ToggleSource {
    async fn read_raw_toggles(env: String) -> EdgeResult<Option<UnleashResponse>>;
}

#[async_trait]
pub trait StatusSink {
    async fn set_status(status: Status) -> EdgeResult<()>;
}

#[async_trait]
pub trait StatusSource {
    async fn get_status() -> EdgeResult<Option<Status>>;
}

pub type EdgeResult<T> = Result<T, EdgeError>;

pub enum EdgeError {}
