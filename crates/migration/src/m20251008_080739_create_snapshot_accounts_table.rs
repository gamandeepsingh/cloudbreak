// SPDX-License-Identifier: AGPL-3.0-only
/*
 * Copyright 2025-2026 Triton One Limited. All rights reserved.
 */

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let connection = manager.get_connection();
        let cfg = crate::migration_config();

        let create_snapshot_accounts_table_sql =
            crate::build_create_table_sql("snapshot_accounts", &cfg.pg_owner_partitions);

        connection
            .execute_unprepared(&create_snapshot_accounts_table_sql)
            .await?;

        // Indexes are being created after the snapshot is processed for performance (snapshot/src/db_queries.rs)

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(SnapshotAccount::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[allow(dead_code)]
#[derive(Iden)]
enum SnapshotAccount {
    #[iden = "snapshot_accounts"]
    Table,
    Pubkey,
    Owner,
    Lamports,
    Slot,
    Executable,
    RentEpoch,
    Data,
    WriteVersion,
    UpdatedOn,
    TxnSignature,
    TokenMint,
    TokenOwner,
}
