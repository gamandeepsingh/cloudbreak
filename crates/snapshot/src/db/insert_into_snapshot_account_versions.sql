-- SPDX-License-Identifier: AGPL-3.0-only
/*
 * Copyright 2025-2026 Triton One Limited. All rights reserved.
 */

INSERT INTO temp_snapshot_account_versions (pubkey, slot, owner)
SELECT
    UNNEST($1::bytea []), -- noqa: AL03
    UNNEST($2::bigint []), -- noqa: AL03
    UNNEST($3::bytea []) -- noqa: AL03
