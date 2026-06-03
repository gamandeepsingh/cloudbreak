// SPDX-License-Identifier: AGPL-3.0-only
/*
 * Copyright 2025-2026 Triton One Limited. All rights reserved.
 */

use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    // This table is going to only have 1 record and it will be used to store the health of the service.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ServiceHealth::Table)
                    .if_not_exists()
                    .col(pk_auto(ServiceHealth::Id))
                    .col(boolean(ServiceHealth::Healthy).not_null())
                    .col(
                        timestamp(ServiceHealth::LastUpdatedAt)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Insert default record
        manager
            .exec_stmt(
                Query::insert()
                    .into_table(ServiceHealth::Table)
                    .columns([ServiceHealth::Healthy])
                    .values_panic([false.into()])
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(ServiceHealth::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum ServiceHealth {
    Table,
    Id,
    Healthy,
    LastUpdatedAt,
}
