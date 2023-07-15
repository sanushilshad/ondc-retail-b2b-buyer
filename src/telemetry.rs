use tracing::{subscriber::set_global_default, Subscriber};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer}; //logging in json format
use tracing_log::LogTracer;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter, Registry};

// use opentelemetry::global;
// use opentelemetry::trace::Tracer;

pub fn get_subscriber<Sink>(
    _name: String,
    env_filter: String,
    sink: Sink,
) -> impl Subscriber + Send + Sync
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    // let tracer = opentelemetry_jaeger::new_agent_pipeline()
    //     .with_endpoint(std::env::var("JAEGER_ENDPOINT").unwrap_or("localhost:4318".to_string()))
    //     .with_service_name("SANU".to_string())
    //     .install_batch(opentelemetry::runtime::Tokio)
    //     .expect("Failed to install OpenTelemetry tracer.");
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(opentelemetry_otlp::new_exporter().tonic())
        .install_batch(opentelemetry::runtime::Tokio)
        .expect("Couldn't create OTLP tracer");
    let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));
    let format_layer = fmt::Layer::default().with_ansi(true).with_writer(sink);
    Registry::default()
        .with(telemetry_layer)
        .with(env_filter)
        .with(format_layer)
}

// Subscriber
pub fn get_json_subscriber<Sink>(
    _name: String,
    env_filter: String,
    sink: Sink,
) -> impl Subscriber + Send + Sync
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));
    let formatting_layer = BunyanFormattingLayer::new(_name, sink);
    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    LogTracer::init().expect("Failed to set logger");
    set_global_default(subscriber).expect("Failed to set subscriber");
}

// #[derive(Debug, Clone)]
// pub struct OpenTelemetryStack {
//     request_metrics: actix_web_opentelemetry::RequestMetrics,
// }

// impl OpenTelemetryStack {
//     pub fn new() -> Self {
//         dotenv().ok();
//         let app_name = std::env::var("CARGO_BIN_NAME").unwrap_or("demo".to_string());

//         global::set_text_map_propagator(opentelemetry_jaeger::Propagator::new());
//         let tracer = opentelemetry_jaeger::new_agent_pipeline()
//             .with_endpoint(std::env::var("JAEGER_ENDPOINT").unwrap_or("localhost:6831".to_string()))
//             .with_service_name(app_name.clone())
//             .install_batch(opentelemetry::runtime::Tokio)
//             .expect("Failed to install OpenTelemetry tracer.");

//         let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
//         let env_filter = EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("INFO"));
//         let formatting_layer = BunyanFormattingLayer::new(app_name.clone().into(), std::io::stdout);
//         let subscriber = Registry::default()
//             .with(telemetry)
//             .with(JsonStorageLayer)
//             .with(formatting_layer)
//             .with(env_filter);
//         tracing::subscriber::set_global_default(subscriber)
//             .expect("Failed to install `tracing` subscriber.");

//         let controller = controllers::basic(processors::factory(
//             sdk::metrics::selectors::simple::histogram([0.1, 0.5, 1.0, 2.0, 5.0, 10.0]),
//             aggregation::cumulative_temporality_selector(),
//         ))
//         .build();
//         let prometheus_exporter = opentelemetry_prometheus::exporter(controller).init();
//         let meter = global::meter("global");
//         let request_metrics = RequestMetricsBuilder::new().build(meter);
//         let metrics_handler = PrometheusMetricsHandler::new(prometheus_exporter.clone());
//         Self {
//             request_metrics,
//             metrics_handler,
//         }
//     }

//     pub fn metrics(&self) -> actix_web_opentelemetry::RequestMetrics {
//         self.request_metrics.clone()
//     }

//     pub fn metrics_handler(&self) -> PrometheusMetricsHandler {
//         self.metrics_handler.clone()
//     }
// }
