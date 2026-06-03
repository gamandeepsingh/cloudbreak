// SPDX-License-Identifier: AGPL-3.0-only
/*
 * Copyright 2025-2026 Triton One Limited. All rights reserved.
 */

use jsonrpsee_types::{ErrorCode, ErrorObject};
use sea_orm::DbErr;

#[derive(thiserror::Error, Debug)]
pub enum QueryTrackerError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] DbErr),
    #[error("Invalid pubkey: {0}")]
    InvalidPubkey(String),
    #[error("Internal error: {0}")]
    InternalError(String),
    #[error("Connection error: {0}")]
    ConnectionError(String),
}

impl From<QueryTrackerError> for ErrorObject<'static> {
    fn from(err: QueryTrackerError) -> Self {
        match err {
            QueryTrackerError::DatabaseError(e) => {
                ErrorObject::owned(ErrorCode::InternalError.code(), e.to_string(), None::<()>)
            }
            QueryTrackerError::InvalidPubkey(msg) => {
                ErrorObject::owned(ErrorCode::InvalidParams.code(), msg, None::<()>)
            }
            QueryTrackerError::InternalError(msg) => {
                ErrorObject::owned(ErrorCode::InternalError.code(), msg, None::<()>)
            }
            QueryTrackerError::ConnectionError(msg) => {
                ErrorObject::owned(ErrorCode::InternalError.code(), msg, None::<()>)
            }
        }
    }
}

pub type QueryTrackerResult<T> = Result<T, QueryTrackerError>;
