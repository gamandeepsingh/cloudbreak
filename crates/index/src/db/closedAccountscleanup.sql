-- SPDX-License-Identifier: AGPL-3.0-only
/*
 * Copyright 2025-2026 Triton One Limited. All rights reserved.
 */

DELETE FROM accounts
WHERE
    pubkey = ANY($1)  -- $1 is array of pubkeys
    AND slot <= $2;     -- $2 is finalized_slot
