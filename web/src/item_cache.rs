use crate::EdgeConfig;
use actix_web::web::Data;
use chrono::Utc;
use std::sync::Arc;
use std::time::Duration;
use storage::{CachedData, FullState, Repository, Status, ToggleSink, TokenStore};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::info;
use unleash_types::client_features::ClientFeatures;

pub(crate) fn init_token_refresher(
    repository: Arc<impl Repository>,
    client: reqwest::Client,
    config: EdgeConfig,
) -> (Arc<FullState>, JoinHandle<()>, CancellationToken) {
    let current_state = Arc::new(repository.init());

    // Stop signal for token refresh job
    let token_refresh_cancel = CancellationToken::new();
    // Spawn refresh job
    (
        Arc::clone(&current_state),
        tokio::spawn(spawn_token_refresh(
            Arc::clone(&current_state),
            client.clone(),
            token_refresh_cancel.clone(),
            config.clone(),
        )),
        token_refresh_cancel,
    )
}

async fn spawn_token_refresh(
    state: Arc<FullState>,
    client: reqwest::Client,
    stop_signal: CancellationToken,
    config: EdgeConfig,
) {
    loop {
        for entry in &state.data {
            let token = entry.key().clone();
            info!("Fetching data for {}", token);
            let updated = fetcher::fetch_client_features(
                client.clone(),
                format!("{}/api/client/features", config.unleash_url.clone()),
                token.clone(),
            )
            .await
            .map(|features| CachedData {
                status: Status {
                    ready: true,
                    last_fetch: Some(Utc::now()),
                    error: None,
                },
                client_features: features.clone(),
            });
            if let Ok(updated_data) = updated {
                info!(
                    "Data was updated. Has {} features",
                    updated_data.client_features.features.len()
                );
                state.data.alter(token.as_str(), |_key, _val| updated_data);
            }
        }
        tokio::select! {
            _ = tokio::time::sleep(Duration::from_secs(config.client_feature_refresh_interval)) => {
                continue;
            }
            _ = stop_signal.cancelled() => {
                info!("gracefully shutting down toggle refresh job");
                break;
            }
        };
    }
}
