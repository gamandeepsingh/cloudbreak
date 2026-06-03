// SPDX-License-Identifier: AGPL-3.0-only
/*
 * Copyright 2025-2026 Triton One Limited. All rights reserved.
 */

use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    Resource,
    trace::{
        BatchConfigBuilder, BatchSpanProcessor, RandomIdGenerator, Sampler, SdkTracerProvider,
    },
};
use serde::Deserialize;
use std::{env, time::Duration};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{
    EnvFilter,
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

// Config
#[derive(Deserialize, Debug, Clone)]
pub struct TracingConfig {
    /// Enable OpenTelemetry export
    #[serde(default)]
    pub enabled: bool,

    /// OTLP endpoint (e.g., "http://localhost:4317")
    #[serde(default = "TracingConfig::default_endpoint")]
    pub endpoint: String,

    /// Sampling ratio (0.0 to 1.0). Default: 0.1 (10%)
    #[serde(
        rename = "sample-ratio",
        default = "TracingConfig::default_sample_ratio"
    )]
    pub sample_ratio: f64,

    /// Batch export interval in seconds
    #[serde(
        rename = "export-interval",
        default = "TracingConfig::default_export_interval"
    )]
    pub export_interval: u64,

    /// Max spans per batch
    #[serde(
        rename = "max-batch-size",
        default = "TracingConfig::default_max_batch_size"
    )]
    pub max_batch_size: usize,

    /// Max queue size before dropping spans
    #[serde(
        rename = "max-queue-size",
        default = "TracingConfig::default_max_queue_size"
    )]
    pub max_queue_size: usize,

    /// Track idle time
    #[serde(
        rename = "track-idle-time",
        default = "TracingConfig::default_track_idle_time"
    )]
    pub track_idle_time: bool,

    /// Span names to export (if empty, export all spans)
    #[serde(rename = "span-filter", default)]
    pub span_filter: Vec<String>,
}

impl TracingConfig {
    fn default_endpoint() -> String {
        "http://localhost:4317".to_string()
    }

    fn default_sample_ratio() -> f64 {
        0.1 // 10% sampling
    }

    /// Batch export interval in seconds
    fn default_export_interval() -> u64 {
        5
    }

    fn default_max_batch_size() -> usize {
        512
    }

    fn default_max_queue_size() -> usize {
        2048
    }

    fn default_track_idle_time() -> bool {
        false
    }

    /// Load the tracing config from the toml file (if not `[tracing]` section is found, it will return the [`Default::default()`] config)
    pub fn try_load(path: &str) -> cloudbreak_core::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let value: toml::Value = toml::from_str(&content)?;

        // Extract just the [tracing] section, or use empty table if missing
        let tracing_section = value
            .get("tracing")
            .cloned()
            .unwrap_or(toml::Value::Table(toml::map::Map::new()));

        Ok(tracing_section.try_into()?)
    }
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            endpoint: Self::default_endpoint(),
            sample_ratio: Self::default_sample_ratio(),
            export_interval: Self::default_export_interval(),
            max_batch_size: Self::default_max_batch_size(),
            max_queue_size: Self::default_max_queue_size(),
            track_idle_time: Self::default_track_idle_time(),
            span_filter: Vec::new(),
        }
    }
}

/// Initialize the tracer (by default only log to stdout with fmt layer, if tracing_config is provided, it will add the OTel layer)
pub fn init_tracer(config: &str) {
    let env_filter = EnvFilter::builder()
        .parse(env::var(EnvFilter::DEFAULT_ENV).unwrap_or_else(|_| "info,sqlx=error".to_owned()))
        .unwrap();

    let (filter_layer, reload_handle) = tracing_subscriber::reload::Layer::new(env_filter);

    // Store the handle globally
    let _ = cloudbreak_core::LOG_FILTER_HANDLE.set(reload_handle);

    let tracing_config = TracingConfig::try_load(config).unwrap_or_default();

    // Only add OTel layer if enabled
    if tracing_config.enabled {
        let fmt_layer = fmt::layer()
            .with_target(false)
            .with_thread_ids(false)
            .with_file(false)
            .with_line_number(false)
            .compact();

        let otel_layer = init_otel_layer(&tracing_config);

        let value_filter = tracing_subscriber::filter::filter_fn(move |metadata| {
            let name_matches = if tracing_config.span_filter.is_empty() {
                true
            } else {
                tracing_config
                    .span_filter
                    .contains(&metadata.name().to_string())
            };

            metadata.is_span() && name_matches
        });

        tracing_subscriber::registry()
            .with(filter_layer)
            .with(tracing_subscriber::filter::Filtered::new(
                otel_layer,
                value_filter,
            ))
            .with(fmt_layer)
            .init();

        tracing::info!(
            target: "opentelemetry",
            "OpenTelemetry tracing enabled (endpoint: {}, sample_ratio: {})",
            tracing_config.endpoint,
            tracing_config.sample_ratio
        );
    } else {
        let fmt_layer = fmt::layer().with_span_events(FmtSpan::CLOSE);
        tracing_subscriber::registry()
            .with(filter_layer)
            .with(fmt_layer)
            .init();
    }
}

/// Initialize the OpenTelemetry layer for tracing
fn init_otel_layer<S>(
    config: &TracingConfig,
) -> OpenTelemetryLayer<S, opentelemetry_sdk::trace::Tracer>
where
    S: tracing::Subscriber + for<'span> tracing_subscriber::registry::LookupSpan<'span>,
{
    // let resource = Resource::builder()
    //     .with_service_name("cloudbreak-api")
    //     .build();

    // Empty resource - no service.name, no telemetry.sdk.*, nothing
    let resource = Resource::builder_empty().build();

    // OTLP exporter with gRPC (most efficient)
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(&config.endpoint)
        .build()
        .expect("Failed to create OTLP exporter");

    // Batch processor config - tuned for low overhead
    let batch_config = BatchConfigBuilder::default()
        .with_max_queue_size(config.max_queue_size)
        .with_max_export_batch_size(config.max_batch_size)
        .with_scheduled_delay(Duration::from_secs(config.export_interval))
        .build();

    let batch_processor = BatchSpanProcessor::builder(exporter)
        .with_batch_config(batch_config)
        .build();

    // Sampler - reduces overhead by only tracing a fraction
    let sampler = if config.sample_ratio >= 1.0 {
        Sampler::AlwaysOn
    } else if config.sample_ratio <= 0.0 {
        Sampler::AlwaysOff
    } else {
        Sampler::TraceIdRatioBased(config.sample_ratio)
    };

    let provider = SdkTracerProvider::builder()
        .with_resource(resource)
        .with_sampler(sampler)
        .with_id_generator(RandomIdGenerator::default())
        .with_span_processor(batch_processor)
        .build();

    let tracer = provider.tracer("cloudbreak");

    opentelemetry::global::set_tracer_provider(provider);

    // Create layer with minimal config
    OpenTelemetryLayer::new(tracer)
        .with_tracked_inactivity(config.track_idle_time) // Sets if track or not track idle time
        .with_target(false)
        .with_location(false)
        .with_threads(false)
}
