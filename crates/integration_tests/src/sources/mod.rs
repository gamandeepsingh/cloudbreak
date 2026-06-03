// SPDX-License-Identifier: AGPL-3.0-only
/*
 * Copyright 2025-2026 Triton One Limited. All rights reserved.
 */

pub mod json_file;
pub mod mismatch_files;
pub mod victoria_logs;

use std::time::Duration;

use crate::{benchmark::RequestType, config::SourceConfig};
use anyhow::Result;
use serde_json::Value as JsonValue;
use tokio::sync::watch;

/// Loads the requests from the source and returns a watch::Receiver that will be updated with new requests depending on the source type
pub async fn load_requests_from_source(
    source: &SourceConfig,
    request_type: RequestType,
) -> Result<watch::Receiver<Vec<JsonValue>>, anyhow::Error> {
    match source {
        SourceConfig::JsonFile { path } => {
            let rx = json_file::load_requests(path)?;

            Ok(rx)
        }
        SourceConfig::VictoriaLogs {
            url,
            minutes,
            limit,
            min_request_size,
            max_request_size,
            encoding,
            inject_context: _,
        } => {
            let client = reqwest::Client::builder()
                .pool_max_idle_per_host(0)
                .build()?;

            tracing::info!(
                target: "bench_source",
                "Fetching initial requests from VictoriaLogs"
            );

            let initial_requests = victoria_logs::get_requests(
                &client,
                url,
                request_type,
                *min_request_size,
                *max_request_size,
                encoding.as_deref(),
                *minutes,
                *limit,
            )
            .await?;

            tracing::info!(
                target: "bench_source",
                "Fetched {} requests from VictoriaLogs",
                initial_requests.len()
            );

            let (tx, rx) = watch::channel(initial_requests);

            // Background fetcher — refreshes every minute
            let url = url.clone();
            let min_request_size = *min_request_size;
            let max_request_size = *max_request_size;
            let encoding = encoding.clone();
            let minutes = *minutes;
            let limit = *limit;

            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(Duration::from_secs(60)).await;
                    match victoria_logs::get_requests(
                        &client,
                        &url,
                        request_type,
                        min_request_size,
                        max_request_size,
                        encoding.as_deref(),
                        minutes,
                        limit,
                    )
                    .await
                    {
                        Ok(new_requests) => {
                            tracing::info!(
                                target: "bench_source",
                                "Refreshed {} requests from VictoriaLogs",
                                new_requests.len()
                            );
                            if tx.send(new_requests).is_err() {
                                break; // all receivers dropped, stop fetching
                            }
                        }
                        Err(e) => {
                            tracing::error!(
                                target: "bench_source",
                                "Failed to refresh requests: {}", e
                            );
                        }
                    }
                }
            });

            Ok(rx)
        }
        SourceConfig::MismatchDir {
            path,
            inject_context,
        } => {
            let rx = mismatch_files::load_requests(path, *inject_context)?;
            Ok(rx)
        }
    }
}
