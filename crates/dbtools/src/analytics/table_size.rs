// SPDX-License-Identifier: AGPL-3.0-only
/*
 * Copyright 2025-2026 Triton One Limited. All rights reserved.
 */

use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, Statement};

/// ```
///┌─────────────────────────────────────────────┐
///│           pg_total_relation_size            │
///│  ┌───────────────────────────────────────┐  │
///│  │           pg_table_size               │  │
///│  │  ┌─────────────────────────────────┐  │  │
///│  │  │      pg_relation_size           │  │  │
///│  │  │         (heap data)             │  │  │
///│  │  └─────────────────────────────────┘  │  │
///│  │  + TOAST + FSM + VM                   │  │
///│  └───────────────────────────────────────┘  │
///│  + INDEXES                                  │
///└─────────────────────────────────────────────┘
/// ```
pub async fn get_table_size(database_url: &str) {
    let db = Database::connect(database_url)
        .await
        .expect("Failed to connect to database");

    // DB size
    get_db_size(&db).await;
    println!();
    println!();
    get_table_size_from_table(&db, "accounts").await;
    println!();
    get_table_size_from_table(&db, "snapshot_accounts").await;
    println!();
    get_single_table_size(&db, "temp_snapshot_account_versions").await;
}

async fn get_db_size(db: &DatabaseConnection) {
    let db_size_query = "SELECT pg_size_pretty(pg_database_size(current_database())) as db_size";
    let rows = db
        .query_all(Statement::from_string(DbBackend::Postgres, db_size_query))
        .await
        .expect("Failed to get database size");

    for row in rows {
        let db_size: String = row.try_get("", "db_size").unwrap();
        println!("=== Database size: {} ===\n", db_size);
    }
}

async fn get_table_size_from_table(db: &DatabaseConnection, table: &str) {
    println!("=== Table: {} ===", table);

    // Handles table partitions
    let size_query = format!(
        r#"
      SELECT 
          COALESCE(SUM(pg_relation_size(c.oid)), 0)::bigint as pg_relation_size,
          COALESCE(SUM(pg_table_size(c.oid)), 0)::bigint as pg_table_size,
          COALESCE(SUM(pg_total_relation_size(c.oid)), 0)::bigint as pg_total_relation_size
      FROM pg_class c
      JOIN pg_inherits i ON c.oid = i.inhrelid
      JOIN pg_class p ON i.inhparent = p.oid
      WHERE p.relname = '{table}'
      "#
    );

    let rows = db
        .query_all(Statement::from_string(DbBackend::Postgres, size_query))
        .await
        .expect("Failed to get table size breakdown");

    let to_mb = 1024 * 1024;

    for row in rows {
        let pg_relation_size: i64 = row.try_get("", "pg_relation_size").unwrap();
        let pg_table_size: i64 = row.try_get("", "pg_table_size").unwrap();
        let pg_total_relation_size: i64 = row.try_get("", "pg_total_relation_size").unwrap();

        println!("pg_relation_size:       {:?} MB", pg_relation_size / to_mb);
        println!("pg_table_size:          {:?} MB", pg_table_size / to_mb);
        println!(
            "pg_total_relation_size: {:?} MB",
            pg_total_relation_size / to_mb
        );

        let indexes_size = pg_total_relation_size - pg_table_size;
        let toast_size = pg_table_size - pg_relation_size;

        println!("indexes_size:           {:?} MB", indexes_size / to_mb);
        println!("toast_size:             {:?} MB", toast_size / to_mb);
    }
}

/// For non-partitioned tables (queries table directly, not via pg_inherits)
async fn get_single_table_size(db: &DatabaseConnection, table: &str) {
    println!("=== Table: {} ===", table);

    let size_query = format!(
        r#"
      SELECT 
          pg_relation_size('{table}')::bigint as pg_relation_size,
          pg_table_size('{table}')::bigint as pg_table_size,
          pg_total_relation_size('{table}')::bigint as pg_total_relation_size
      "#
    );

    let rows = db
        .query_all(Statement::from_string(DbBackend::Postgres, size_query))
        .await
        .expect("Failed to get table size breakdown");

    let to_mb = 1024 * 1024;

    for row in rows {
        let pg_relation_size: i64 = row.try_get("", "pg_relation_size").unwrap();
        let pg_table_size: i64 = row.try_get("", "pg_table_size").unwrap();
        let pg_total_relation_size: i64 = row.try_get("", "pg_total_relation_size").unwrap();

        println!("pg_relation_size:       {} MB", pg_relation_size / to_mb);
        println!("pg_table_size:          {} MB", pg_table_size / to_mb);
        println!(
            "pg_total_relation_size: {} MB",
            pg_total_relation_size / to_mb
        );

        let indexes_size = pg_total_relation_size - pg_table_size;
        let toast_size = pg_table_size - pg_relation_size;

        println!("indexes_size:           {} MB", indexes_size / to_mb);
        println!("toast_size:             {} MB", toast_size / to_mb);
    }
}
