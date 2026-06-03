// SPDX-License-Identifier: AGPL-3.0-only
/*
 * Copyright 2025-2026 Triton One Limited. All rights reserved.
 */

use sea_orm::{ConnectionTrait, Database, DbBackend, Statement};

pub async fn get_slow_queries(database_url: &str) {
    let db = Database::connect(database_url)
        .await
        .expect("Failed to connect to database");

    let sql = r#"
        SELECT 
            pid,
            (now() - pg_stat_activity.query_start)::text AS duration,
            state,
            query
        FROM pg_stat_activity
        WHERE query NOT ILIKE '%pg_stat_activity%'
          AND query_start IS NOT NULL
          AND state IS NOT NULL
          AND now() - pg_stat_activity.query_start > interval '1 minute'
        ORDER BY duration DESC;
              "#;
    let slow_queries = db
        .query_all(Statement::from_string(DbBackend::Postgres, sql))
        .await
        .expect("Failed to get slow queries");

    for row in slow_queries {
        let pid: i32 = row.try_get("", "pid").unwrap_or(0);
        let duration: String = row.try_get("", "duration").unwrap_or_default();
        let state: String = row.try_get("", "state").unwrap_or_default();
        let query: String = row.try_get("", "query").unwrap_or_default();
        println!(
            "PID: {}, Duration: {}, State: {}, Query: {}",
            pid, duration, state, query
        );
    }
}
