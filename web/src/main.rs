use actix_web::web::{Data, Json};
use actix_web::{middleware, web, App, HttpServer};
use actix_web_opentelemetry::{PrometheusMetricsHandler, RequestMetricsBuilder, RequestTracing};
use clap::Parser;
use opentelemetry::global;
use opentelemetry::sdk::export::metrics::aggregation;
use opentelemetry::sdk::metrics::{controllers, processors, selectors};
use tracing::info;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};

mod backstage;
use types::EdgeError;

mod proxy;
mod telemetry;

pub type EdgeJsonResult<T> = Result<Json<T>, EdgeError>;

#[derive(Parser)]
pub struct EdgeFeatures {
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
}

#[tokio::main]
async fn main() -> Result<(), EdgeError> {
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
    let toggle_source = Data::new(storage::memory::InMemoryRepository::default());
    // Parse Unleash Edge Options
    let args = EdgeFeatures::parse();
    let server = HttpServer::new(move || {
        App::new()
            .wrap(RequestTracing::new())
            .wrap(request_metrics.clone())
            .wrap(middleware::Logger::default().exclude("/internal-backstage"))
            .app_data(toggle_source.clone())
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
        _ = server.run() => info!("Actix has now exited")
    }
    Ok(())
}
