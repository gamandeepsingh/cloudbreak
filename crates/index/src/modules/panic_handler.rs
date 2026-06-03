// SPDX-License-Identifier: AGPL-3.0-only
/*
 * Copyright 2025-2026 Triton One Limited. All rights reserved.
 */

/// Catches potentially silent panics inside not awaited tokio spawned tasks.
pub fn start() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        // run the default hook so the existing logging / location info is preserved
        default_hook(info);
        // give async log writers a moment to flush
        std::thread::sleep(std::time::Duration::from_millis(200));
        std::process::exit(1);
    }));
}
