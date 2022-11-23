use types::{EdgeError, EdgeResult};
use unleash_types::client_features::ClientFeatures;

async fn fetch_client_features(
    client: reqwest::Client,
    unleash_url: String,
    unleash_token: String,
) -> EdgeResult<ClientFeatures> {
    let result = client
        .get(unleash_url)
        .header("Authorization", unleash_token)
        .send()
        .await
        .map_err(|_| EdgeError::UnleashApiError)?;
    result
        .json::<ClientFeatures>()
        .await
        .map_err(|_| EdgeError::UnleashJsonError)
}
