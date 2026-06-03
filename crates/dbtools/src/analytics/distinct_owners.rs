// SPDX-License-Identifier: AGPL-3.0-only
/*
 * Copyright 2025-2026 Triton One Limited. All rights reserved.
 */

use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, Statement};

pub async fn get_distinct_owners_count(database_url: &str) {
    let db = Database::connect(database_url)
        .await
        .expect("Failed to connect to database");

    // get_distinct_owners_from_table(&db, "accounts").await;
    // get_distinct_owners_from_table(&db, "snapshot_accounts").await;

    get_distinct_owners_from_table_by_partition(&db, "accounts").await;
    get_distinct_owners_from_table_by_partition(&db, "snapshot_accounts").await;
}

pub async fn get_distinct_owners_from_table(db: &DatabaseConnection, table: &str) {
    let distinct_owners = db
        .query_all(Statement::from_string(
            DbBackend::Postgres,
            format!("SELECT COUNT(DISTINCT owner) FROM {table}"),
        ))
        .await
        .expect("Failed to get distinct owners");

    println!(
        "Table: {table} (Distinct owners: {})",
        distinct_owners
            .first()
            .unwrap()
            .try_get::<i64>("", "count")
            .unwrap()
    );
}

async fn get_distinct_owners_from_table_by_partition(db: &DatabaseConnection, table: &str) {
    // Total distinct owners
    let mut total_distinct_owners = 0;

    // Distinct owners per partition
    let per_partition = db
        .query_all(Statement::from_string(
            DbBackend::Postgres,
            format!(
                r#"
              SELECT 
                  tableoid::regclass::text as partition_name,
                  COUNT(DISTINCT owner) as distinct_owners
              FROM {table}
              GROUP BY tableoid
              ORDER BY distinct_owners DESC
              "#
            ),
        ))
        .await
        .expect("Failed to get distinct owners per partition");

    println!("\n  Distinct owners by partition:");
    for row in per_partition {
        let partition_name: String = row.try_get("", "partition_name").unwrap();
        let count: i64 = row.try_get("", "distinct_owners").unwrap();
        total_distinct_owners += count;

        println!("    {:<50} {:>10}", partition_name, count);
    }
    println!();
    println!(
        "Table: {table} (Total distinct owners: {})",
        total_distinct_owners
    );
    println!("########################################################\n\n");
}
