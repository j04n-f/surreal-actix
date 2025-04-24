use crate::config::{LoggingConfig, ServiceConfig};

use opentelemetry::trace::{TraceError, TracerProvider};
use opentelemetry::{KeyValue, global};
use opentelemetry_sdk::{
    Resource, error::OTelSdkError, propagation::TraceContextPropagator, trace::SdkTracerProvider,
};
use opentelemetry_semantic_conventions::resource;
use thiserror::Error;
use tracing::subscriber::SetGlobalDefaultError;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{EnvFilter, Registry, filter::LevelFilter, layer::SubscriberExt};

#[derive(Error, Debug)]
pub enum OTelError {
    #[error(transparent)]
    Subscriber(#[from] SetGlobalDefaultError),
    #[error(transparent)]
    OTelSdk(#[from] OTelSdkError),
    #[error(transparent)]
    Trace(#[from] TraceError),
}

pub fn configure(
    service_config: &ServiceConfig,
    logging_config: &LoggingConfig,
) -> Result<SdkTracerProvider, OTelError> {
    global::set_text_map_propagator(TraceContextPropagator::new());

    let otlp_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .build()?;

    let resource = Resource::builder()
        .with_attribute(KeyValue::new(
            resource::SERVICE_NAME,
            service_config.name.to_owned(),
        ))
        .build();

    let provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_batch_exporter(otlp_exporter)
        .with_resource(resource)
        .build();

    let tracer = provider.tracer(service_config.name.to_owned());

    let env_filter = EnvFilter::new(logging_level(&logging_config.level));

    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    let formatting_layer =
        BunyanFormattingLayer::new(service_config.name.to_owned(), std::io::stdout);

    let subscriber = Registry::default()
        .with(env_filter)
        .with(telemetry)
        .with(JsonStorageLayer)
        .with(formatting_layer);

    tracing::subscriber::set_global_default(subscriber)?;

    Ok(provider)
}

pub fn shutdown(provider: SdkTracerProvider) -> Result<(), OTelError> {
    Ok(provider.shutdown()?)
}

fn logging_level(level: &str) -> String {
    let filter = match level {
        "off" => LevelFilter::OFF,
        "debug" => LevelFilter::DEBUG,
        "trace" => LevelFilter::TRACE,
        "info" => LevelFilter::INFO,
        "warn" => LevelFilter::WARN,
        "error" => LevelFilter::ERROR,
        _ => LevelFilter::ERROR,
    };

    filter.to_string()
}
