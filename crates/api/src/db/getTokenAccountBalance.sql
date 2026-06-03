-- SPDX-License-Identifier: AGPL-3.0-only
/*
 * Copyright 2025-2026 Triton One Limited. All rights reserved.
 */

WITH all_versions AS (
    SELECT
        accounts.pubkey,
        accounts.owner,
        accounts.lamports,
        accounts.slot,
        accounts.data,
        accounts.token_mint
    FROM accounts
    WHERE
        accounts.pubkey = $1
        AND accounts.slot <= $2
    UNION ALL
    SELECT
        snapshot_accounts.pubkey,
        snapshot_accounts.owner,
        snapshot_accounts.lamports,
        snapshot_accounts.slot,
        snapshot_accounts.data,
        snapshot_accounts.token_mint
    FROM snapshot_accounts
    WHERE
        snapshot_accounts.pubkey = $1
        AND snapshot_accounts.slot <= $2
),

latest_account AS (
    SELECT
        pubkey,
        owner,
        lamports,
        slot,
        data,
        token_mint
    FROM all_versions
    ORDER BY slot DESC
    LIMIT 1
),

needed_mint AS (
    SELECT token_mint AS mint_pubkey
    FROM latest_account
    WHERE
        lamports > 0
        AND token_mint IS NOT NULL
        AND (
            owner = '\x06ddf6e1d765a193d9cbe146ceeb79ac1cb485ed5f5b37913a8cf5857eff00a9'::bytea -- TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA -- noqa: LT05
            OR owner = '\x06ddf6e1ee758fde18425dbce46ccddab61afc4d83b90d27febdf928d8a18bfc'::bytea -- TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb -- noqa: LT05
        )
),

all_mint_versions AS NOT MATERIALIZED (
    SELECT
        accounts.pubkey,
        accounts.data,
        accounts.slot,
        accounts.lamports
    FROM accounts
    INNER JOIN needed_mint ON accounts.pubkey = needed_mint.mint_pubkey
    WHERE accounts.slot <= $2
    UNION ALL
    SELECT
        snapshot_accounts.pubkey,
        snapshot_accounts.data,
        snapshot_accounts.slot,
        snapshot_accounts.lamports
    FROM snapshot_accounts
    INNER JOIN needed_mint ON snapshot_accounts.pubkey = needed_mint.mint_pubkey
    WHERE snapshot_accounts.slot <= $2
),

mint AS (
    -- Same filter-after-DISTINCT-ON pattern as getAccountInfoWithMintData.sql.
    SELECT
        pubkey,
        mint_data
    FROM (
        SELECT DISTINCT ON (pubkey)
            pubkey,
            data AS mint_data,
            lamports
        FROM all_mint_versions
        ORDER BY pubkey ASC, slot DESC
    ) AS latest_per_pubkey
    WHERE lamports > 0
)

SELECT
    latest_account.owner,
    latest_account.token_mint,
    mint.mint_data,
    CASE
        WHEN
            latest_account.owner = '\x06ddf6e1d765a193d9cbe146ceeb79ac1cb485ed5f5b37913a8cf5857eff00a9'::bytea -- noqa: LT05
            OR latest_account.owner = '\x06ddf6e1ee758fde18425dbce46ccddab61afc4d83b90d27febdf928d8a18bfc'::bytea -- noqa: LT05
            THEN SUBSTRING(latest_account.data FROM 65 FOR 8)
        ELSE '\x0000000000000000'::bytea
    END AS amount
FROM latest_account
LEFT JOIN mint ON latest_account.token_mint = mint.pubkey
WHERE latest_account.lamports > 0;
