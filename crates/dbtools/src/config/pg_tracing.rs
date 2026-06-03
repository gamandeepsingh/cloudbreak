// SPDX-License-Identifier: AGPL-3.0-only
/*
 * Copyright 2025-2026 Triton One Limited. All rights reserved.
 */

use sea_orm::{ConnectionTrait, Database, DbBackend, Statement};

pub async fn pg_tracing_high_detail(database_url: &str, enable: bool) {
    let db = Database::connect(database_url)
        .await
        .expect("Failed to connect to database");

    let value = if enable { "true" } else { "false" };

    let statements = vec![
        format!("ALTER SYSTEM SET pg_tracing.planstate_spans = {value};"),
        // format!("ALTER SYSTEM SET pg_tracing.deparse_plan = {value};"),
        "SELECT pg_reload_conf();".to_string(),
    ];

    for sql in &statements {
        db.execute(Statement::from_string(DbBackend::Postgres, sql))
            .await
            .unwrap_or_else(|e| panic!("Failed to execute '{}': {}", sql, e));
        println!("Executed: {}", sql);
    }

    println!(
        "pg_tracing high detail has been {}.",
        if enable { "enabled" } else { "disabled" }
    );
}
