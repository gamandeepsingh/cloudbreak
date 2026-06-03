-- SPDX-License-Identifier: AGPL-3.0-only
/*
 * Copyright 2025-2026 Triton One Limited. All rights reserved.
 */

WITH
all_mints AS (
    SELECT
        data,
        slot
    FROM accounts
    WHERE
        owner = $3
        AND pubkey = $1
        AND slot <= $2
    UNION ALL
    SELECT
        data,
        slot
    FROM snapshot_accounts
    WHERE
        owner = $3
        AND pubkey = $1
        AND slot <= $2
)

SELECT data
FROM all_mints
ORDER BY slot DESC LIMIT 1;
