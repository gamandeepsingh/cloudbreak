// SPDX-License-Identifier: AGPL-3.0-only
/*
 * Copyright 2025-2026 Triton One Limited. All rights reserved.
 */

use crate::{
    error::RpcError,
    http::{CloudbreakApiResponse, CloudbreakRpcState},
};

#[tracing::instrument(name = "getGenesisHash", skip_all)]
pub async fn get_genesis_hash(
    state: &CloudbreakRpcState,
) -> Result<CloudbreakApiResponse<String>, RpcError> {
    Ok(CloudbreakApiResponse::Response(state.genesis_hash.clone()))
}
