-- SPDX-License-Identifier: AGPL-3.0-only
/*
 * Copyright 2025-2026 Triton One Limited. All rights reserved.
 */

DELETE FROM snapshot_accounts
USING accounts_to_delete
WHERE
    snapshot_accounts.owner = accounts_to_delete.owner
    AND snapshot_accounts.pubkey = accounts_to_delete.pubkey
    AND snapshot_accounts.slot = accounts_to_delete.slot;
