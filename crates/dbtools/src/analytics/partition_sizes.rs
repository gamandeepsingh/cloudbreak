// SPDX-License-Identifier: AGPL-3.0-only
/*
 * Copyright 2025-2026 Triton One Limited. All rights reserved.
 */

use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, Statement};

pub async fn get_partition_sizes(database_url: &str) {
    let db = Database::connect(database_url)
        .await
        .expect("Failed to connect to database");

    get_partition_sizes_from_table(&db, "accounts").await;
    get_partition_sizes_from_table(&db, "snapshot_accounts").await;
}

async fn get_partition_sizes_from_table(db: &DatabaseConnection, table: &str) {
    let query = format!(
        r#"
    WITH RECURSIVE partition_tree AS (
        SELECT 
            c.oid,
            c.relname as partition_name,
            1 as level,
            p.relname as parent_name
        FROM pg_class c
        JOIN pg_inherits i ON c.oid = i.inhrelid
        JOIN pg_class p ON i.inhparent = p.oid
        WHERE p.relname = '{table}'
        
        UNION ALL
        
        SELECT 
            c.oid,
            c.relname as partition_name,
            pt.level + 1 as level,
            pt.partition_name as parent_name
        FROM pg_class c
        JOIN pg_inherits i ON c.oid = i.inhrelid
        JOIN partition_tree pt ON i.inhparent = pt.oid
    )
    SELECT 
        partition_name,
        level,
        parent_name,
        pg_size_pretty(pg_total_relation_size(oid)) as total_size,
        pg_total_relation_size(oid) as size_bytes
    FROM partition_tree
    ORDER BY parent_name, level, size_bytes DESC
    "#
    );

    let rows = db
        .query_all(Statement::from_string(DbBackend::Postgres, query))
        .await
        .expect("Failed to get partition sizes");

    println!("=== Table {} ===\n", table);
    println!(
        "{:<60} | {:>5} | {:<40} | {:>12} | {:>15}",
        "partition name", "level", "parent", "total size", "size bytes"
    );
    println!("{}", "-".repeat(140));
    for row in rows {
        let partition_name: String = row.try_get("", "partition_name").unwrap();
        let level: i32 = row.try_get("", "level").unwrap();
        let parent_name: String = row.try_get("", "parent_name").unwrap();
        let total_size: String = row.try_get("", "total_size").unwrap();
        let size_bytes: i64 = row.try_get("", "size_bytes").unwrap();

        let indent = "  ".repeat((level - 1) as usize);
        println!(
            "{:<60} | {:>5} | {:<40} | {:>12} | {:>15}",
            format!("{}{}", indent, partition_name),
            level,
            parent_name,
            total_size,
            size_bytes
        );
    }
    println!("{}\n\n", "-".repeat(140));
}
