// SPDX-License-Identifier: AGPL-3.0-only
/*
 * Copyright 2025-2026 Triton One Limited. All rights reserved.
 */

use sea_orm::{ConnectionTrait, Database, DbBackend, Statement};
use serde_json::json;
use solana_pubkey::Pubkey;

const SQL: &str = r#"
SELECT 
    pubkey,
    owner,
    token_mint,
    SUBSTRING(data FROM 77 FOR 32) as delegate_pubkey
FROM snapshot_accounts
WHERE (owner = '\x06ddf6e1d765a193d9cbe146ceeb79ac1cb485ed5f5b37913a8cf5857eff00a9'::bytea
    OR owner = '\x06ddf6e1ee758fde18425dbce46ccddab61afc4d83b90d27febdf928d8a18bfc'::bytea)
AND SUBSTRING(data FROM 73 FOR 1) = '\x01'::bytea
LIMIT 10;
"#;

const DISTINCT_TOTAL_DELEGATES: &str = r#"
SELECT 
    COUNT(*) as total_delegated_accounts,
    COUNT(DISTINCT SUBSTRING(data FROM 77 FOR 32)) as unique_delegates
FROM snapshot_accounts
WHERE (owner = '\x06ddf6e1d765a193d9cbe146ceeb79ac1cb485ed5f5b37913a8cf5857eff00a9'::bytea
    OR owner = '\x06ddf6e1ee758fde18425dbce46ccddab61afc4d83b90d27febdf928d8a18bfc'::bytea)
AND SUBSTRING(data FROM 73 FOR 1) = '\x01'::bytea;
"#;

pub async fn get_delegates(database_url: &str) {
    let db = Database::connect(database_url)
        .await
        .expect("Failed to connect to database");

    let delegates = db
        .query_all(Statement::from_string(DbBackend::Postgres, SQL))
        .await
        .expect("Failed to get delegates");
    let distinct_total_delegates = db
        .query_all(Statement::from_string(
            DbBackend::Postgres,
            DISTINCT_TOTAL_DELEGATES,
        ))
        .await
        .expect("Failed to get distinct total delegates");

    let mut request_bodies = Vec::new();

    for delegate in delegates {
        let pubkey_bytes: Vec<u8> = delegate.try_get("", "pubkey").unwrap();
        let delegate_bytes: Vec<u8> = delegate.try_get("", "delegate_pubkey").unwrap();
        let owner_bytes: Vec<u8> = delegate.try_get("", "owner").unwrap();

        let pubkey = Pubkey::try_from(pubkey_bytes.as_slice()).unwrap();
        let delegate_pubkey = Pubkey::try_from(delegate_bytes.as_slice()).unwrap();
        let owner = Pubkey::try_from(owner_bytes.as_slice()).unwrap();
        let token_mint: Vec<u8> = delegate.try_get("", "token_mint").unwrap();
        let token_mint = Pubkey::try_from(token_mint.as_slice()).unwrap();

        println!(
            "Pubkey: {}, Delegate pubkey: {}, Owner: {}, Token mint: {}",
            pubkey, delegate_pubkey, owner, token_mint
        );

        // Build request body in the expected format
        request_bodies.push(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getTokenAccountsByDelegate",
            "params": [
                delegate_pubkey.to_string(),
                {
                    "mint": token_mint.to_string()
                },
                {
                    "encoding": "jsonParsed"
                }
            ]
        }));
    }

    // Save to file
    std::fs::write(
        "crates/integration_tests/compare_responses_results/gtabd_bodies.json",
        serde_json::to_string_pretty(&request_bodies).expect("Failed to serialize"),
    )
    .expect("Failed to write file");

    for distinct_total_delegate in distinct_total_delegates {
        let total_delegated_accounts: i64 = distinct_total_delegate
            .try_get("", "total_delegated_accounts")
            .unwrap();
        let unique_delegates: i64 = distinct_total_delegate
            .try_get("", "unique_delegates")
            .unwrap();
        println!(
            "Total delegated accounts: {}, Unique delegates: {}",
            total_delegated_accounts, unique_delegates
        );
    }
}
