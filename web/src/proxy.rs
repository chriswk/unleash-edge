use std::collections::HashMap;
use actix_web::{web, get, HttpRequest};
use actix_web::web::{Data, Json};
use sdk_core::EngineState;
use sdk_core::state::InnerContext;
use unleash_types::client_features::Payload;
use unleash_types::frontend::{EvaluatedToggle, EvaluatedVariant, FrontendResult};
use storage::ToggleSource;
use types::{EdgeError, EdgeToken};
use crate::EdgeJsonResult;
use serde::{Deserialize};
use tracing::info;
use storage::memory::InMemoryRepository;

#[derive(Debug, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct QueryData {
    user_id: Option<String>,
    session_id: Option<String>,
    environment: Option<String>,
    app_name: Option<String>,
    properties: Option<HashMap<String, String>>
}


async fn evaluate_toggles(token: EdgeToken, client_features_provider: Data<InMemoryRepository>, req: HttpRequest) -> EdgeJsonResult<FrontendResult> {
    let unleash_context = web::Query::<QueryData>::from_query(req.query_string())
        .map_err(|_| EdgeError::CouldNotParseQuery)
        .map(|q| {
        let query = q.into_inner();
        InnerContext {
            user_id: query.user_id,
            session_id: query.session_id,
            environment: query.environment,
            app_name: query.app_name,
            current_time: Some(chrono::Utc::now().to_rfc3339()),
            remote_address: Some(req.connection_info().host().to_string()),
            properties: None,
        }
    })
        .unwrap_or(InnerContext::default());
    info!("Requested with context: {:#?}", unleash_context);
    client_features_provider
        .get_ref()
        .read_raw_toggles(&token.environment)
        .await
        .map(|client_features| {
            if let Some(features) = client_features {
            let mut state = EngineState::new();
            state.take_state(features.clone());
            let evaluated_toggles: Vec<EvaluatedToggle> = features.features.into_iter().map(|toggle| {
                let variant = state.get_variant(toggle.name.clone(), &unleash_context);
                EvaluatedToggle {
                    name: toggle.name.clone(),
                    enabled: state.is_enabled(toggle.name, &unleash_context),
                    variant: EvaluatedVariant {
                        name: variant.name,
                        enabled: variant.enabled,
                        payload: variant.payload.map(|succ| {
                            Payload {
                                payload_type: succ.payload_type,
                                value: succ.value,
                            }
                        })
                    },
                    impression_data: false,
                }
            }).collect();
            FrontendResult {
                toggles: evaluated_toggles
            }
        } else {
            FrontendResult {
                toggles: vec![]
            }
        }
    }).map(Json)
}

pub fn configure_proxy(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/toggles").route(web::get().to(evaluate_toggles)));
}
