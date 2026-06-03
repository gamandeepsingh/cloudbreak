-- SPDX-License-Identifier: AGPL-3.0-only
/*
 * Copyright 2025-2026 Triton One Limited. All rights reserved.
 */

WITH
    all_accounts AS NOT MATERIALIZED (
        SELECT
            pubkey,
            owner,
            slot
        FROM accounts
        WHERE pubkey = ANY($1)
        UNION ALL
        SELECT
            pubkey,
            owner,
            slot
        FROM snapshot_accounts
        WHERE pubkey = ANY($1)
    ),
    latest_owner AS NOT MATERIALIZED (
        SELECT DISTINCT ON (pubkey)
            pubkey,
            owner
        FROM all_accounts
        ORDER BY pubkey ASC, slot DESC
    )
INSERT INTO accounts (pubkey, owner, lamports, slot, executable, rent_epoch, data, write_version)
SELECT
    latest_owner.pubkey,
    latest_owner.owner,
    0,  -- lamports
    $2,  -- slot (same for all accounts in the batch)
    FALSE,
    0,
    '\x'::bytea,  -- data: empty bytes
    0
FROM latest_owner;
