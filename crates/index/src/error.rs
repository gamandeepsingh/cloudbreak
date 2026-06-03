// SPDX-License-Identifier: AGPL-3.0-only
/*
 * Copyright 2025-2026 Triton One Limited. All rights reserved.
 */

use std::fmt::Debug;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProcessorError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sea_orm::DbErr),
    #[error("Transaction error: {0}")]
    TransactionError(#[from] sea_orm::TransactionError<sea_orm::DbErr>),
    #[error("Query error: {0}")]
    QueryError(#[from] sea_orm::sea_query::error::Error),
    #[error("Channel send error: {0}")]
    ChannelSendError(String),
    #[error("Process Join error")]
    JoinError(#[from] tokio::task::JoinError),
}
