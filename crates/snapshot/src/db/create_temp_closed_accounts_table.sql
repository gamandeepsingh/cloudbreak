-- SPDX-License-Identifier: AGPL-3.0-only
/*
 * Copyright 2025-2026 Triton One Limited. All rights reserved.
 */

CREATE UNLOGGED TABLE IF NOT EXISTS temp_closed_accounts (pubkey BYTEA PRIMARY KEY, slot BIGINT NOT NULL);
