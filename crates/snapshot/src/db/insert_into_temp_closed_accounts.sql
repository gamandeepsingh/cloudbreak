-- SPDX-License-Identifier: AGPL-3.0-only
/*
 * Copyright 2025-2026 Triton One Limited. All rights reserved.
 */

INSERT INTO temp_closed_accounts (pubkey, slot)
SELECT
    UNNEST($1::bytea []) AS pubkey,
    UNNEST($2::bigint []) AS slot;
