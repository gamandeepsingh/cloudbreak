// SPDX-License-Identifier: AGPL-3.0-only
/*
 * Copyright 2025-2026 Triton One Limited. All rights reserved.
 */

use serde_json::Value as JsonValue;
use tokio::sync::watch;

/// The Json file can contain any type of valid request and it will be sent as it is to the RPC endpoints
pub fn load_requests(path: &str) -> Result<watch::Receiver<Vec<JsonValue>>, anyhow::Error> {
    let file_content = std::fs::read_to_string(path)?;
    let requests: Vec<JsonValue> = serde_json::from_str(&file_content)?;

    // _tx is dropped — JsonFile is static, no updates
    let (_tx, rx) = watch::channel(requests);

    Ok(rx)
}
