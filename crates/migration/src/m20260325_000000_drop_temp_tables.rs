// SPDX-License-Identifier: AGPL-3.0-only
/*
 * Copyright 2025-2026 Triton One Limited. All rights reserved.
 */

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let connection = manager.get_connection();

        connection
            .execute_unprepared("DROP TABLE IF EXISTS temp_snapshot_account_versions;")
            .await?;

        connection
            .execute_unprepared("DROP TABLE IF EXISTS accounts_to_delete;")
            .await?;

        connection
            .execute_unprepared("DROP TABLE IF EXISTS temp_closed_accounts;")
            .await?;

        Ok(())
    }
}
