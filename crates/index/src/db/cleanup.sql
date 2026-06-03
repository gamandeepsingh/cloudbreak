-- SPDX-License-Identifier: AGPL-3.0-only
/*
 * Copyright 2025-2026 Triton One Limited. All rights reserved.
 */

-- EXPLAIN ANALYZE
DELETE FROM accounts_table_name  -- placeholder to be replaced with the actual table name
WHERE
    pubkey = ANY($2)  -- $2 is array of pubkeys
    AND slot < $1     -- $1 is finalized_slot
    -- tecnically this is not needed, because if we keep track of the updated accounts, if there is 
    -- any older version we can safely delete it (so this is just an extra check)
    -- AND EXISTS (
    --     SELECT 1 FROM accounts AS a2
    --     WHERE
    --         a2.pubkey = accounts.pubkey
    --         AND a2.slot > accounts.slot
    --         AND a2.slot < $1
    -- );
