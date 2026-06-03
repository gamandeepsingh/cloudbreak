// SPDX-License-Identifier: AGPL-3.0-only
/*
 * Copyright 2025-2026 Triton One Limited. All rights reserved.
 */

use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, Statement};

pub async fn get_indexes_sizes(database_url: &str) {
    let db = Database::connect(database_url)
        .await
        .expect("Failed to connect to database");

    get_indexes_sizes_from_partitioned_table(&db, "accounts").await;
    println!();
    get_indexes_sizes_from_partitioned_table(&db, "snapshot_accounts").await;
}

#[allow(unused)]
async fn get_indexes_sizes_from_table(db: &DatabaseConnection, table: &str) {
    let index_query = format!(
        r#"
      SELECT 
          indexname,
          pg_size_pretty(pg_relation_size(indexname::regclass)) as index_size
      FROM pg_indexes 
      WHERE tablename = '{table}'
      ORDER BY pg_relation_size(indexname::regclass) DESC
      "#
    );

    let rows = db
        .query_all(Statement::from_string(DbBackend::Postgres, index_query))
        .await
        .expect("Failed to get index sizes");

    if !rows.is_empty() {
        println!("\n  Indexes:");
        for row in rows {
            let indexname: String = row.try_get("", "indexname").unwrap();
            let index_size: String = row.try_get("", "index_size").unwrap();
            println!("    - {}: {}", indexname, index_size);
        }
    }
}

async fn get_indexes_sizes_from_partitioned_table(db: &DatabaseConnection, table: &str) {
    // Sum index sizes across all partitions recursively (handles subpartitions)
    let index_query = format!(
        r#"
      WITH RECURSIVE all_partitions AS (
          -- Base case: direct children
          SELECT c.oid
          FROM pg_class c
          JOIN pg_inherits i ON c.oid = i.inhrelid
          JOIN pg_class p ON i.inhparent = p.oid
          WHERE p.relname = '{table}'
          
          UNION ALL
          
          -- Recursive case: subpartitions
          SELECT c.oid
          FROM pg_class c
          JOIN pg_inherits i ON c.oid = i.inhrelid
          JOIN all_partitions ap ON i.inhparent = ap.oid
      )
      SELECT 
          regexp_replace(idx.indexrelid::regclass::text, '_p\d+', '') as index_name,
          pg_size_pretty(SUM(pg_relation_size(idx.indexrelid))::bigint) as total_size,
          SUM(pg_relation_size(idx.indexrelid))::bigint as size_bytes
      FROM pg_index idx
      JOIN all_partitions ap ON idx.indrelid = ap.oid
      GROUP BY regexp_replace(idx.indexrelid::regclass::text, '_p\d+', '')
      ORDER BY SUM(pg_relation_size(idx.indexrelid)) DESC
      "#
    );

    let rows = db
        .query_all(Statement::from_string(DbBackend::Postgres, index_query))
        .await
        .expect("Failed to get index sizes");

    if !rows.is_empty() {
        println!("=== Indexes for {} ===", table);
        for row in rows {
            let index_name: String = row.try_get("", "index_name").unwrap();
            let total_size: String = row.try_get("", "total_size").unwrap();
            println!("  - {}: {}", index_name, total_size);
        }
    }
}
