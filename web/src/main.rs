use actix_web::web::{Data, Json};
use actix_web::{middleware, web, App, HttpServer};
use actix_web_opentelemetry::{PrometheusMetricsHandler, RequestMetricsBuilder, RequestTracing};
use clap::Parser;
use opentelemetry::global;
use opentelemetry::sdk::export::metrics::aggregation;
use opentelemetry::sdk::metrics::{controllers, processors, selectors};
use reqwest::ClientBuilder;
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};

mod backstage;
use types::EdgeError;

mod item_cache;
mod proxy;
mod telemetry;

pub type EdgeJsonResult<T> = Result<Json<T>, EdgeError>;

#[derive(Parser, Debug, Clone)]
pub struct EdgeConfig {
    /// URL to use to connect to Unleash API Server or another Unleash Edge server
    #[clap(short, long, env)]
    pub unleash_url: String,

    /// A list of client tokens (Optional) that is preapproved
    #[clap(short, long, env)]
    pub tokens: Vec<String>,

    /// Which port should Edge bind to
    #[clap(short, long, env)]
    pub port: Option<u16>,

    /// Which ip should Edge bind to
    #[clap(short, long, env, default_value = "0.0.0.0")]
    pub ip: Option<String>,

    /// How often to refresh features for a client key (in seconds)
    #[clap(short, long, env, default_value_t = 15)]
    pub client_feature_refresh_interval: u64,
}

#[tokio::main]
async fn main() -> Result<(), EdgeError> {
    // Dotenv
    dotenv::dotenv().ok();
    // Parse Unleash Edge Options
    let args = EdgeConfig::parse();

    #[cfg(feature = "telemetry")]
    let telemetry = tracing_opentelemetry::layer().with_tracer(telemetry::init_tracer());
    let logger = tracing_subscriber::fmt::layer();
    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    // Decide on layers
    #[cfg(feature = "telemetry")]
    let collector = Registry::default()
        .with(telemetry)
        .with(logger)
        .with(env_filter);
    #[cfg(not(feature = "telemetry"))]
    let collector = Registry::default().with(logger).with(env_filter);

    // Initialize tracing
    tracing::subscriber::set_global_default(collector).unwrap();
    let metrics_handler = {
        let controller = controllers::basic(
            processors::factory(
                selectors::simple::histogram([1.0, 2.0, 5.0, 10.0, 20.0, 50.0]),
                aggregation::cumulative_temporality_selector(),
            )
            .with_memory(true),
        )
        .build();

        let exporter = opentelemetry_prometheus::exporter(controller).init();
        PrometheusMetricsHandler::new(exporter)
    };
    let meter = global::meter("actix_web");
    let request_metrics = RequestMetricsBuilder::new().build(meter);

    // Configure refreshing of data
    let toggle_source = Arc::new(storage::memory::InMemoryRepository::default());
    let http_client = ClientBuilder::new()
        .build()
        .map_err(|_| EdgeError::NoHttpClient)?;
    let (toggle_cache, toggle_refresher, toggle_refresh_cancel) =
        item_cache::init_token_refresher(toggle_source.clone(), http_client.clone(), args.clone());
    let server = HttpServer::new(move || {
        App::new()
            .wrap(RequestTracing::new())
            .wrap(request_metrics.clone())
            .wrap(middleware::Logger::default().exclude("/internal-backstage"))
            .app_data(Data::new(toggle_source.clone()))
            .app_data(Data::new(toggle_cache.clone()))
            .service(
                web::resource("/internal-backstage/metrics")
                    .route(web::get().to(metrics_handler.clone())),
            )
            .service(web::scope("/internal-backstage").configure(backstage::configure_backstage))
            .service(web::scope("/api").configure(proxy::configure_proxy))
    })
    .bind((
        args.ip.unwrap_or("0.0.0.0".into()),
        args.port.unwrap_or(3001),
    ))
    .map_err(|_| EdgeError::CouldNotBind)
    .expect("Can not bind")
    .shutdown_timeout(5);
    tokio::select! {
        _ = server.run() => {
            toggle_refresh_cancel.cancel();
            toggle_refresher.await.unwrap();
            info!("Actix has now exited");
        }
    }
    Ok(())
}
