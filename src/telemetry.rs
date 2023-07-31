use tracing::{subscriber::set_global_default, Subscriber};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer}; //logging in json format
use tracing_log::LogTracer;
// use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::fmt::MakeWriter;
// use tracing_subscriber::Layer;
use tracing_subscriber::{
    filter::LevelFilter, fmt, layer::SubscriberExt, EnvFilter, Layer, Registry,
};
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
    let format_layer = fmt::Layer::default()
        .with_ansi(true)
        .with_writer(sink)
        .with_filter(LevelFilter::DEBUG);
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
