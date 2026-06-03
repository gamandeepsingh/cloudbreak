// SPDX-License-Identifier: AGPL-3.0-only
/*
 * Copyright 2025-2026 Triton One Limited. All rights reserved.
 */

use sea_orm::{ConnectionTrait, Database, Statement};

const LEGACY_TOKEN_PROGRAM: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
const TOKEN_2022_PROGRAM: &str = "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb";

const ACCOUNT_TYPE_MINT: u8 = 1;

pub async fn get_mint_accounts_count(database_url: &str) {
    get_mint_accounts_count_for_table(database_url, "accounts").await;
    get_mint_accounts_count_for_table(database_url, "snapshot_accounts").await;
}

pub async fn get_mint_accounts_count_for_table(database_url: &str, table: &str) {
    let db = Database::connect(database_url)
        .await
        .expect("Failed to connect to database");

    // Convert program IDs to hex for SQL
    let legacy_program_hex = hex::encode(
        LEGACY_TOKEN_PROGRAM
            .parse::<solana_pubkey::Pubkey>()
            .unwrap()
            .to_bytes(),
    );
    let token22_program_hex = hex::encode(
        TOKEN_2022_PROGRAM
            .parse::<solana_pubkey::Pubkey>()
            .unwrap()
            .to_bytes(),
    );

    // 1. Legacy Token Program mints (owner = tokenkeg, len = 82)
    let legacy_mints_query = format!(
        r#"
        SELECT COUNT(*) as count FROM {table} 
        WHERE owner = '\x{}'::bytea 
          AND length(data) = 82
        "#,
        legacy_program_hex
    );

    // 2. Token-2022 base mints without extensions (owner = token22, len = 82)
    let token22_base_mints_query = format!(
        r#"
        SELECT COUNT(*) as count FROM {table} 
        WHERE owner = '\x{}'::bytea 
          AND length(data) = 82
        "#,
        token22_program_hex
    );

    // 3. Token-2022 mints with extensions (owner = token22, len > 165, byte 165 = 1)
    let token22_ext_mints_query = format!(
        r#"
        SELECT COUNT(*) as count FROM {table} 
        WHERE owner = '\x{}'::bytea 
          AND length(data) > 165
          AND get_byte(data, 165) = {}
        "#,
        token22_program_hex, ACCOUNT_TYPE_MINT
    );

    // Execute queries
    let legacy_count: i64 = db
        .query_one(Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            legacy_mints_query,
        ))
        .await
        .expect("Query failed")
        .map(|r| r.try_get_by_index::<i64>(0).unwrap_or(0))
        .unwrap_or(0);

    let token22_base_count: i64 = db
        .query_one(Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            token22_base_mints_query,
        ))
        .await
        .expect("Query failed")
        .map(|r| r.try_get_by_index::<i64>(0).unwrap_or(0))
        .unwrap_or(0);

    let token22_ext_count: i64 = db
        .query_one(Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            token22_ext_mints_query,
        ))
        .await
        .expect("Query failed")
        .map(|r| r.try_get_by_index::<i64>(0).unwrap_or(0))
        .unwrap_or(0);

    println!("=== Mint Accounts Count for table: {table} ===");
    println!(
        "Legacy Token Program mints (82 bytes):     {}",
        legacy_count
    );
    println!(
        "Token-2022 base mints (82 bytes):          {}",
        token22_base_count
    );
    println!(
        "Token-2022 mints with extensions (>165):   {}",
        token22_ext_count
    );
    println!("---");
    println!(
        "Total mints:                               {}",
        legacy_count + token22_base_count + token22_ext_count
    );
}
