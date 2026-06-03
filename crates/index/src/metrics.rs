// SPDX-License-Identifier: AGPL-3.0-only
/*
 * Copyright 2025-2026 Triton One Limited. All rights reserved.
 */

use std::{
    collections::HashMap,
    convert::Infallible,
    net::SocketAddr,
    sync::{Arc, Mutex, Once, OnceLock},
};

use http_body_util::Full;
use hyper::{
    Request, Response,
    body::{Bytes, Incoming},
    service::service_fn,
};
use hyper_util::{rt::TokioIo, server::conn::auto};

use prometheus::{
    Counter, Histogram, HistogramOpts, HistogramVec, IntGauge, IntGaugeVec, Opts, Registry,
    TextEncoder,
};
use tokio::net::TcpListener;
use tracing::{error, info};
use cloudbreak_core::{IndexConfig, modules::account_owner_map::AccountOwnerMap};

use crate::{
    indexer::AccountsReceivedPerBlock,
    modules::self_healing::{MissingSlot, SlotGap},
};

pub static DB_ERRORS_THRESHOLD: OnceLock<f64> = OnceLock::new();

lazy_static::lazy_static! {
    pub static ref METRICS_REGISTRY: Registry = Registry::new();

    pub static ref BLOCK_PROCESSING: HistogramVec = HistogramVec::new(
        HistogramOpts::new(
            "cloudbreak_block_processing",
            "cloudbreak_block_processing"
        )
        .buckets(vec![
            0.01, // 10ms
            0.05, // 50ms
            0.1, // 100ms
            0.15, // 150ms
            0.2, // 200ms
            0.25, // 250ms
            0.3, // 300ms
            0.35, // 350ms
            0.4, // 400ms
            0.45, // 450ms
            0.5, // 500ms
            1.0, // 1s
            10.0, // 10s
        ]),
        &["origin"]
    ).unwrap();

    pub static ref CHUNK_PROCESSING: HistogramVec = HistogramVec::new(
        HistogramOpts::new(
            "cloudbreak_chunk_processing",
            "cloudbreak_chunk_processing"
        )
        .buckets(vec![
            0.01, // 10ms
            0.05, // 50ms
            0.1, // 100ms
            0.15, // 150ms
            0.2, // 200ms
            0.25, // 250ms
            0.3, // 300ms
            0.35, // 350ms
            0.4, // 400ms
            0.45, // 450ms
            0.5, // 500ms
            1.0, // 1s
            10.0, // 10s
        ]),
        &["origin"]
    ).unwrap();

    pub static ref BLOCK_SIZE_HISTOGRAM: HistogramVec = HistogramVec::new(
        HistogramOpts::new("cloudbreak_block_size", "Size of blocks in bytes")
            .buckets(vec![
                100_000.0,    // 100KB
                500_000.0,    // 500KB
                1_000_000.0,  // 1MB
                5_000_000.0,  // 5MB
                10_000_000.0, // 10MB
                15_000_000.0, // 15MB
                20_000_000.0, // 20MB
                30_000_000.0, // 30MB
            ]),
        &["origin"],
    )
    .expect("Failed to create block size histogram");

    pub static ref CHUNK_SIZE_HISTOGRAM: HistogramVec = HistogramVec::new(
        HistogramOpts::new("cloudbreak_chunk_size", "Size of chunks in bytes")
            .buckets(vec![
                50_000.0,     // 50KB
                100_000.0,
                200_000.0,
                300_000.0,
                400_000.0,
                500_000.0,
                600_000.0,
                700_000.0,
                800_000.0,
                1_000_000.0,
                2_000_000.0,  // 2MB
            ]),
        &["origin"],
    )
    .expect("Failed to create block size histogram");

    pub static ref FINALIZE_SLOT: HistogramVec = HistogramVec::new(
        HistogramOpts::new(
            "cloudbreak_finalize_slot",
            "cloudbreak_finalize_slot"
        ).buckets(vec![
            0.005,
            0.01, // 10ms
            0.02,
            0.025,
            0.03,
            0.04,
            0.05,
            0.075,
            0.1, // 100ms
            0.15,
            0.2,
            0.25,
            0.35,
            0.5,
            1.0,
            10.0,
        ]),
        &["origin"]
    ).unwrap();

    pub static ref FINALIZE_SLOT_DELETED_ACCOUNTS: Histogram = Histogram::with_opts(
        HistogramOpts::new(
            "cloudbreak_finalize_slot_deleted_accounts",
            "cloudbreak_finalize_slot_deleted_accounts"
        ).buckets(vec![
            1.0,
            10.0,
            100.0,
            300.0,
            400.0,
            500.0,
            600.0,
            700.0,
            1000.0,
            2000.0,
            5000.0,
            10000.0,
        ]),
    ).unwrap();

    pub static ref NEW_ACCOUNTS_IN_SLOT_HISTOGRAM: HistogramVec = HistogramVec::new(
        HistogramOpts::new("cloudbreak_new_accounts_in_slot", "Number of brand new accounts in a slot")
            .buckets(vec![
                100.0,
                200.0,
                300.0,
                400.0,
                500.0,
                600.0,
                700.0,
                800.0,
                900.0,
                1000.0,
                1100.0,
                1200.0,
                1300.0,
                1400.0,
                1500.0,
            ]),
        &["origin"],
    )
    .expect("Failed to create block size histogram");

    pub static ref GRPC_BUFFER_CHANNEL_SIZE_HISTOGRAM: HistogramVec = HistogramVec::new(
        HistogramOpts::new("cloudbreak_grpc_buffer_channel_size", "Size of the GRPC buffer channel")
            .buckets(vec![
                1.0,
                5.0,
                10.0,
                25.0,
                50.0,
                100.0,
                200.0,
                300.0,
                400.0,
                500.0,
                600.0,
                700.0,
                800.0,
                900.0,
                1000.0,
                1100.0,
                10_000.0,
            ]),
        &["origin"],
    )
    .expect("Failed to create block size histogram");

    pub static ref GRPC_BUFFER_CHANNEL_SIZE_SENDER: IntGauge = IntGauge::new(
        "cloudbreak_grpc_buffer_channel_size_sender", "Size of the GRPC buffer channel sender"
    )
    .expect("Failed to create GRPC buffer channel size sender gauge");

    pub static ref GRPC_TIMEOUT_ERRORS: Counter = Counter::new(
        "cloudbreak_grpc_timeout_errors", "Number of GRPC timeout errors"
    )
    .expect("Failed to create GRPC timeout errors counter");

    pub static ref GRPC_ERRORS: Counter = Counter::new(
        "cloudbreak_grpc_errors", "Number of GRPC errors"
    )
    .expect("Failed to create GRPC errors counter");

    pub static ref GRPC_TOTAL_UPDATES_RECEIVED: Counter = Counter::new(
        "cloudbreak_grpc_total_updates_received", "Number of GRPC updates received"
    )
    .expect("Failed to create GRPC total updates received counter");

    pub static ref GRPC_GAP_ERRORS: Counter = Counter::new(
        "cloudbreak_grpc_gap_errors", "Number of slot gaps on grpc reconnection"
    )
    .expect("Failed to create GRPC gap errors counter");

    pub static ref DB_ERRORS: Counter = Counter::new(
        "cloudbreak_db_errors", "Number of DB errors"
    )
    .expect("Failed to create DB errors counter");

    pub static ref CLOSED_ACCOUNTS_PER_SLOT_HISTOGRAM: Histogram = Histogram::with_opts(
        HistogramOpts::new("cloudbreak_closed_accounts_per_slot", "Number of closed accounts per slot")
            .buckets(vec![
                1.0,
                5.0,
                10.0,
                25.0,
                50.0,
                100.0,
                200.0,
                300.0,
                400.0,
                500.0,
                600.0,
                700.0,
                800.0,
                900.0,
                1000.0,
                1100.0,
                10_000.0,
            ]),
    )
    .expect("Failed to create closed accounts per slot histogram");

    pub static ref INSERT_CLOSED_ACCOUNTS_PER_SLOT_HISTOGRAM: Histogram = Histogram::with_opts(
        HistogramOpts::new("cloudbreak_insert_closed_accounts_per_slot_ms", "Latency of inserting closed accounts per slot in milliseconds")
            .buckets(vec![
                0.1,
                1.0,
                5.0,
                10.0,
                25.0,
                50.0,
                100.0,
                200.0,
                300.0,
                400.0,
                500.0,
                600.0,
                700.0,
                800.0,
                900.0,
                1000.0,
                10_000.0,
            ]),
    )
    .expect("Failed to create insert closed accounts per slot histogram");

    pub static ref CURRENT_TOKIO_TASKS: IntGaugeVec = IntGaugeVec::new(
        Opts::new("cloudbreak_current_tokio_tasks", "Current number of Tokio tasks"),
        &["task_type"],
    )
    .expect("Failed to create current tokio tasks histogram");

    pub static ref FINALIZE_SLOT_HANDLER_QUEUE_SIZE: IntGauge = IntGauge::new(
        "cloudbreak_finalize_slot_handler_queue_size", "Size of the finalize slot handler queue"
    )
    .expect("Failed to create finalize slot handler queue size gauge");
}

/// We use a guard to increment the current tokio tasks metric when a task is created and
///  decrement it when the task is dropped. This way the counter is going to be decremented
/// even in the case of panics.
pub struct TokioTaskCounterGuard {
    task_type: String,
}

impl TokioTaskCounterGuard {
    pub fn new(task_type: &str) -> Self {
        CURRENT_TOKIO_TASKS.with_label_values(&[task_type]).inc();
        Self {
            task_type: task_type.to_string(),
        }
    }

    pub fn decrement(&self) {
        CURRENT_TOKIO_TASKS
            .with_label_values(&[&self.task_type])
            .dec();
    }
}

impl Drop for TokioTaskCounterGuard {
    fn drop(&mut self) {
        self.decrement();
    }
}

pub fn record_block_processing(elapsed: f64, origin: &str) {
    BLOCK_PROCESSING
        .with_label_values(&[origin])
        .observe(elapsed);
}

pub fn record_chunk_processing(elapsed: f64, origin: &str) {
    CHUNK_PROCESSING
        .with_label_values(&[origin])
        .observe(elapsed);
}

pub fn record_block_size(size: usize) {
    BLOCK_SIZE_HISTOGRAM
        .with_label_values(&["block"])
        .observe(size as f64);
}

pub fn record_chunk_size(size: usize) {
    CHUNK_SIZE_HISTOGRAM
        .with_label_values(&["chunk"])
        .observe(size as f64);
}

pub fn record_finalize_slot(elapsed: f64, tag: &str) {
    FINALIZE_SLOT.with_label_values(&[tag]).observe(elapsed);
}

pub fn record_new_accounts_in_slot(count: usize, tag: &str) {
    NEW_ACCOUNTS_IN_SLOT_HISTOGRAM
        .with_label_values(&[tag])
        .observe(count as f64);
}

/// Keep track of how big is the GRPC buffer channel
pub fn record_grpc_buffer_channel_size(size: usize) {
    GRPC_BUFFER_CHANNEL_SIZE_HISTOGRAM
        .with_label_values(&["grpc_buffer_channel_size"])
        .observe(size as f64);
}

pub fn increment_grpc_timeout_errors() {
    GRPC_TIMEOUT_ERRORS.inc();
}

pub fn increment_grpc_errors() {
    GRPC_ERRORS.inc();
}

pub fn increment_grpc_gap_errors() {
    GRPC_GAP_ERRORS.inc();
}

pub fn increment_db_errors() {
    DB_ERRORS.inc();

    let threshold = DB_ERRORS_THRESHOLD.get().copied().unwrap_or(100.0);
    if threshold > 0.0 && DB_ERRORS.get() > threshold {
        error!("DB errors threshold reached: {}", DB_ERRORS.get());
        std::process::exit(1);
    }
}

pub fn record_closed_accounts_per_slot(count: usize) {
    CLOSED_ACCOUNTS_PER_SLOT_HISTOGRAM.observe(count as f64);
}

fn metrics_handler() -> Result<Response<Full<Bytes>>, Infallible> {
    let metrics = TextEncoder::new()
        .encode_to_string(&METRICS_REGISTRY.gather())
        .unwrap_or_else(|error| {
            error!("could not encode custom metrics: {error}");
            String::new()
        });

    Ok(Response::builder()
        .header("content-type", "text/plain")
        .body(Full::new(Bytes::from(metrics)))
        .unwrap())
}

async fn handle_metrics_request(
    req: Request<Incoming>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    match req.uri().path() {
        "/metrics" => metrics_handler(),
        "/debug/updated_accounts_map" => debug_updated_accounts_map_handler(),
        "/debug/gaps_list" => debug_gaps_list_handler(),
        "/debug/accounts_owner_map" => debug_accounts_owner_map_handler(),
        _ => Ok(not_found_handler()),
    }
}

fn not_found_handler() -> Response<Full<Bytes>> {
    Response::builder()
        .status(404)
        .body(Full::new(Bytes::from("Not Found")))
        .unwrap()
}

pub fn run_metrics_server(address: SocketAddr) -> anyhow::Result<()> {
    static REGISTER: Once = Once::new();

    REGISTER.call_once(|| {
        macro_rules! register {
            ($collector:ident) => {
                METRICS_REGISTRY
                    .register(Box::new($collector.clone()))
                    .expect("collector can't be registered");
            };
        }

        register!(BLOCK_PROCESSING);
        register!(CHUNK_PROCESSING);
        register!(BLOCK_SIZE_HISTOGRAM);
        register!(CHUNK_SIZE_HISTOGRAM);
        register!(FINALIZE_SLOT);
        register!(NEW_ACCOUNTS_IN_SLOT_HISTOGRAM);
        register!(GRPC_BUFFER_CHANNEL_SIZE_HISTOGRAM);
        register!(GRPC_TIMEOUT_ERRORS);
        register!(GRPC_ERRORS);
        register!(GRPC_GAP_ERRORS);
        register!(DB_ERRORS);
        register!(CLOSED_ACCOUNTS_PER_SLOT_HISTOGRAM);
        register!(INSERT_CLOSED_ACCOUNTS_PER_SLOT_HISTOGRAM);
        register!(CURRENT_TOKIO_TASKS);
        register!(FINALIZE_SLOT_HANDLER_QUEUE_SIZE);
        register!(GRPC_TOTAL_UPDATES_RECEIVED);
        register!(GRPC_BUFFER_CHANNEL_SIZE_SENDER);
        register!(FINALIZE_SLOT_DELETED_ACCOUNTS);
    });

    tokio::spawn(async move {
        let _guard = TokioTaskCounterGuard::new("metrics_server");

        let listener = match TcpListener::bind(address).await {
            Ok(l) => {
                info!("Prometheus server started at http://{address}/metrics");
                l
            }
            Err(e) => {
                error!("Failed to bind Prometheus server: {e:?}");
                return;
            }
        };

        loop {
            let (stream, _) = match listener.accept().await {
                Ok(pair) => pair,
                Err(e) => {
                    error!("Prometheus accept failed: {e:?}");
                    continue;
                }
            };

            let io = TokioIo::new(stream);
            let service = service_fn(move |req: Request<Incoming>| handle_metrics_request(req));

            tokio::spawn(async move {
                let _guard = TokioTaskCounterGuard::new("metrics_server");

                let builder = auto::Builder::new(hyper_util::rt::TokioExecutor::new());
                let conn = builder.serve_connection(io, service);
                if let Err(e) = conn.await {
                    error!("Prometheus connection failed: {e:?}");
                }
            });
        }
    });

    Ok(())
}

pub fn setup_metrics(config: &IndexConfig) -> anyhow::Result<()> {
    DB_ERRORS_THRESHOLD
        .set(config.database.max_db_errors_threshold.unwrap_or(100.0))
        .ok();

    run_metrics_server(config.get_prom_metrics_collector_endpoint())
}

pub static UPDATED_ACCOUNTS_MAP: OnceLock<Arc<Mutex<HashMap<u64, AccountsReceivedPerBlock>>>> =
    OnceLock::new();
pub static GAPS_TO_CONFIRM: OnceLock<Arc<Mutex<Vec<SlotGap>>>> = OnceLock::new();
pub static GAPS_LIST: OnceLock<Arc<Mutex<Vec<MissingSlot>>>> = OnceLock::new();

fn debug_updated_accounts_map_handler() -> Result<Response<Full<Bytes>>, Infallible> {
    let body = match UPDATED_ACCOUNTS_MAP.get() {
        Some(map) => {
            let map = map.lock().expect("Failed to lock");
            let slots = map
                .keys()
                .map(|slot| slot.to_string())
                .collect::<Vec<_>>()
                .join("\n");

            format!("updated_accounts_map ({} slots):\n\n{}", map.len(), slots)
        }
        None => "Map not initialized".to_string(),
    };

    Ok(Response::builder()
        .header("content-type", "text/plain")
        .body(Full::new(Bytes::from(body)))
        .unwrap())
}

fn debug_gaps_list_handler() -> Result<Response<Full<Bytes>>, Infallible> {
    let body = match (GAPS_LIST.get(), GAPS_TO_CONFIRM.get()) {
        (Some(gaps), Some(gaps_to_confirm)) => {
            let gaps = gaps.lock().expect("Failed to lock");
            let gaps_to_confirm = gaps_to_confirm.lock().expect("Failed to lock");

            let gaps_to_confirm_lines = gaps_to_confirm
                .iter()
                .map(|g| format!("start_slot: {} - end_slot: {}", g.start_slot, g.end_slot))
                .collect::<Vec<_>>()
                .join("\n");

            let gaps_lines = gaps
                .iter()
                .map(|g| format!("slot {} (confirmed: {})", g.slot, g.is_confirmed))
                .collect::<Vec<_>>()
                .join("\n");

            format!(
                "gaps_list ({} slots):\n\n{}\n\ngaps_to_confirm ({} slots):\n\n{}",
                gaps.len(),
                gaps_lines,
                gaps_to_confirm.len(),
                gaps_to_confirm_lines
            )
        }
        _ => "Not initialized".to_string(),
    };

    Ok(Response::builder()
        .header("content-type", "text/plain")
        .body(Full::new(Bytes::from(body)))
        .unwrap())
}

fn debug_accounts_owner_map_handler() -> Result<Response<Full<Bytes>>, Infallible> {
    let body = AccountOwnerMap::debug_accounts_owner_map();

    Ok(Response::builder()
        .header("content-type", "text/plain")
        .body(Full::new(Bytes::from(body)))
        .unwrap())
}
