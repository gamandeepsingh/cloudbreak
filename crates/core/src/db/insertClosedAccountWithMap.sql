-- SPDX-License-Identifier: AGPL-3.0-only
/*
 * Copyright 2025-2026 Triton One Limited. All rights reserved.
 */

INSERT INTO accounts (
    pubkey, owner, lamports, slot, executable, rent_epoch, data, write_version
)
SELECT
    UNNEST($1::bytea []) AS pubkey,
    UNNEST($2::bytea []) AS account_owner,
    0 AS lamports,
    $3 AS slot,
    FALSE AS executable,
    0 AS rent_epoch,
    '\x'::bytea AS account_data,
    0 AS write_version;
