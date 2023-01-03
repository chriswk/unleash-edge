use actix_web::web::{Data, Json};
use actix_web::{middleware, web, App, HttpServer};
use clap::Parser;
use tracing::info;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};
mod backstage;
use types::EdgeError;
mod telemetry;
mod proxy;

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
    // Parse Unleash Edge Options
    let args = EdgeFeatures::parse();
    let server = HttpServer::new(move || {
        let toggle_source = storage::memory::InMemoryRepository::default();
        App::new()
            .app_data(Data::new(toggle_source))
            .wrap(middleware::Logger::default().exclude("/internal-backstage"))
            .service(web::scope("/internal-backstage").configure(backstage::configure_backstage))
            .service(web::scope("/api").configure(proxy::configure_proxy))
    })
    .bind(("0.0.0.0", args.port.unwrap_or(3001)))
    .map_err(|_| EdgeError::CouldNotBind)
    .expect("Can not bind")
    .shutdown_timeout(5);
    tokio::select! {
        _ = server.run() => info!("Actix has now exited")
    }
    Ok(())
}
