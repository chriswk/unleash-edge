use tokio::sync::mpsc::Receiver;
use tracing::info;
use types::{EdgeError, EdgeResult, EdgeToken};
use unleash_types::client_features::ClientFeatures;

pub async fn fetch_client_features(
    client: reqwest::Client,
    unleash_url: String,
    unleash_token: String,
) -> EdgeResult<ClientFeatures> {
    let result = client
        .get(unleash_url)
        .header("Authorization", unleash_token.clone())
        .send()
        .await
        .map_err(|_| EdgeError::UnleashApiError)?;
    info!("Successfully fetched data for token: {}", unleash_token);
    let data = result
        .json::<ClientFeatures>()
        .await
        .map_err(|_| EdgeError::UnleashJsonError);
    info!("converted to json: {:#?}", data);
    data
}
