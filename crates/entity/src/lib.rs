// SPDX-License-Identifier: AGPL-3.0-only
/*
 * Copyright 2025-2026 Triton One Limited. All rights reserved.
 */

use sea_orm::prelude::*;

mod generated;
pub use generated::*;

#[derive(Debug, thiserror::Error)]
pub enum AccountConversionError {
    #[error("Db error: {0}")]
    Database(#[from] sea_orm::DbErr),
    #[error("Numeric conversion error")]
    ConversionFailed,
    #[error("Invalid Pubkey")]
    Pubkey,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum CommitmentLevel {
    #[sea_orm(num_value = 0)]
    Processed,
    #[sea_orm(num_value = 1)]
    Confirmed,
    #[sea_orm(num_value = 2)]
    Finalized,
}
