// SPDX-License-Identifier: AGPL-3.0-only
/*
 * Copyright 2025-2026 Triton One Limited. All rights reserved.
 */

use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, Statement};

pub async fn get_indexes_count(database_url: &str) {
    let db = Database::connect(database_url)
        .await
        .expect("Failed to connect to database");

    let accounts_indexes = get_indexes_count_from_table(&db, "accounts").await;
    let snapshot_indexes = get_indexes_count_from_table(&db, "snapshot_accounts").await;

    println!("Indexes count for accounts: {}", accounts_indexes);
    println!("Indexes count for snapshot_accounts: {}", snapshot_indexes);
}

async fn get_indexes_count_from_table(db: &DatabaseConnection, table: &str) -> i64 {
    let indexes = db
        .query_all(Statement::from_string(
            DbBackend::Postgres,
            format!("SELECT COUNT(*) FROM pg_indexes WHERE tablename = '{table}'"),
        ))
        .await
        .expect("Failed to get indexes count");
    indexes
        .first()
        .unwrap()
        .try_get::<i64>("", "count")
        .unwrap()
}
