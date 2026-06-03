// SPDX-License-Identifier: AGPL-3.0-only
/*
 * Copyright 2025-2026 Triton One Limited. All rights reserved.
 */

use sea_orm::DatabaseConnection;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use tokio::{sync::mpsc::Sender, task::JoinSet, time::Instant};
use yellowstone_grpc_proto::geyser::CommitmentLevel;
use cloudbreak_core::IndexConfig;

use crate::indexer::AccountsReceivedPerBlock;
use crate::modules::snapshot::SnapshotProcessingState;
use crate::{db_queries, metrics};

const SLOT_FINALIZE_BATCH_SIZE: usize = 500;

/// Receives a vector of the accounts that were updated in the slot and deletes "older" versions
///  of those accounts
async fn finalize_slot(
    config: &IndexConfig,
    slot: u64,
    db: DatabaseConnection,
    updated_accounts: AccountsReceivedPerBlock,
    updated_accounts_during_startup: UpdatedAccountsDuringStartup,
    pending_gap_fill_replays: PendingGapFillReplays,
) {
    let start_time = Instant::now();

    let db_clone = db.clone();
    let config_clone = config.clone();

    // Mark the slot as finalized before starting the cleanup tasks for API queries consistency
    db_queries::insert_slot(
        slot,
        updated_accounts.block_time,
        CommitmentLevel::Finalized,
        &db_clone,
        &config_clone,
    )
    .await;

    // These are accounts that were in the slot but did not have an older version (which means
    //  they are completely new to our db)
    let new_accounts_in_slot = Arc::new(Mutex::new(0));

    let batches = updated_accounts
        .accounts
        .chunks(SLOT_FINALIZE_BATCH_SIZE)
        .map(|batch| batch.to_vec())
        .collect::<Vec<_>>();

    let mut join_set = JoinSet::new();

    updated_accounts_during_startup.cleanup_stored_accounts_once(&db, slot, config);

    for batch in batches {
        let db_clone = db.clone();
        let batch_clone = batch.clone();
        let new_accounts_in_slot_clone = new_accounts_in_slot.clone();
        let updated_accounts_during_startup = updated_accounts_during_startup.clone();
        let config_clone = config.clone();
        join_set.spawn(async move {
            let _guard = metrics::TokioTaskCounterGuard::new("finalize_slot_internal");

            db_queries::cleanup_accounts(
                &db_clone,
                batch_clone,
                slot,
                "accounts",
                new_accounts_in_slot_clone,
                "cleanup_accounts_batch",
                &config_clone,
            )
            .await;
        });

        // If we are in startup, we just save the updated accounts to delete them after the snapshot is processed
        if updated_accounts_during_startup.is_startup() {
            updated_accounts_during_startup.add_batch_to_cache_during_startup(batch);
            continue;
        }

        let db_clone = db.clone();
        let config_clone = config.clone();
        join_set.spawn(async move {
            let _guard = metrics::TokioTaskCounterGuard::new("finalize_slot_internal");

            // with the latest changes it doesn't make sense any more to try to measure this on the snapshot accounts table
            // but this will asintotically become more accurate as the snapshot accounts table is deleted/cleaned up
            let dummy_new_accounts_in_slot = Arc::new(Mutex::new(0));

            db_queries::cleanup_accounts(
                &db_clone,
                batch,
                slot,
                "snapshot_accounts",
                dummy_new_accounts_in_slot,
                "cleanup_snapshot_accounts_batch",
                &config_clone,
            )
            .await;
        });
    }

    let gap_fill_active = *pending_gap_fill_replays
        .gap_fill_active
        .lock()
        .expect("Failed to lock gap_fill_active");
    if gap_fill_active && !updated_accounts.closed_accounts.is_empty() {
        pending_gap_fill_replays
            .closures_buffer
            .lock()
            .expect("Failed to lock closures_buffer")
            .push(SlotAccounts {
                accounts: updated_accounts.closed_accounts.clone(),
                slot,
            });
    }

    let closed_accounts = updated_accounts.closed_accounts.clone();
    let db_clone = db.clone();
    let config_clone = config.clone();
    join_set.spawn(async move {
        // Updated accounts doesn't include the closed accounts, instead this query will delete the closed accounts inserted
        //  and any previous version of the accounts, so it's safe to execute concurrently with the cleanup_accounts tasks
        // because there is not overlap between the accounts sets
        db_queries::cleanup_closed_accounts(&db_clone, closed_accounts, slot, &config_clone).await;
    });

    // If we are in startup, we just save the closed accounts to delete them after the snapshot is processed
    if updated_accounts_during_startup.is_startup() {
        updated_accounts_during_startup
            .add_batch_to_cache_during_startup(updated_accounts.closed_accounts);
    } else {
        let config_clone = config.clone();
        join_set.spawn(async move {
            // Closed accounts are not included in the updated accounts, so we need to cleanup them separately
            db_queries::cleanup_accounts(
                &db,
                updated_accounts.closed_accounts,
                slot,
                "snapshot_accounts",
                Arc::new(Mutex::new(0)),
                "cleanup_snapshot_closed_accounts",
                &config_clone,
            )
            .await;
        });
    }

    join_set.join_all().await;

    metrics::record_finalize_slot(start_time.elapsed().as_secs_f64(), "total");
    metrics::record_new_accounts_in_slot(
        *new_accounts_in_slot
            .lock()
            .expect("Failed to lock new_accounts_in_slot"),
        "new_accounts_in_slot",
    );
}

///Used to store all accounts that are updated/closed while loading the snapshot, and delete them after the snapshot is processed
#[derive(Clone)]
pub struct UpdatedAccountsDuringStartup {
    pub accounts: Arc<Mutex<HashSet<Vec<u8>>>>,
    pub snapshot_processing_state: Arc<Mutex<SnapshotProcessingState>>,
}

impl UpdatedAccountsDuringStartup {
    pub fn new(snapshot_processing_state: Arc<Mutex<SnapshotProcessingState>>) -> Self {
        Self {
            accounts: Arc::new(Mutex::new(HashSet::new())),
            snapshot_processing_state,
        }
    }

    pub fn is_startup(&self) -> bool {
        let snapshot_processing_state = self
            .snapshot_processing_state
            .lock()
            .expect("Failed to lock snapshot_processing_state");
        *snapshot_processing_state == SnapshotProcessingState::NotStarted
            || *snapshot_processing_state == SnapshotProcessingState::Started
    }

    pub fn add_batch_to_cache_during_startup(&self, batch: Vec<Vec<u8>>) {
        let mut accounts = self.accounts.lock().expect("Failed to lock accounts");
        accounts.extend(batch);
    }

    /// Only cleans up the accounts if we are NOT in startup and if the accounts cache is not empty already
    fn cleanup_stored_accounts_once(
        &self,
        db: &DatabaseConnection,
        slot: u64,
        config: &IndexConfig,
    ) {
        if self.is_startup()
            || self
                .accounts
                .lock()
                .expect("Failed to lock accounts")
                .is_empty()
        {
            return;
        }

        let accounts = self
            .accounts
            .lock()
            .expect("Failed to lock accounts")
            .drain()
            .collect::<Vec<_>>();

        let db = db.clone();
        let config = config.clone();
        let snapshot_processing_state = self.snapshot_processing_state.clone();

        tokio::spawn(async move {
            let _guard = metrics::TokioTaskCounterGuard::new("startup_snapshot_accounts_cleanup");

            let start_time = Instant::now();

            tracing::info!(target: "cleanup_stored_accounts", "Cleaning up stored accounts from snapshot_accounts - accounts: {}", accounts.len());

            let batches = accounts
                .chunks(SLOT_FINALIZE_BATCH_SIZE)
                .map(|batch| batch.to_vec())
                .collect::<Vec<_>>();

            let mut join_set = JoinSet::new();
            const MAX_CONCURRENT_CLEANUP_TASKS: usize = 10;

            for batch in batches {
                while join_set.len() >= MAX_CONCURRENT_CLEANUP_TASKS {
                    join_set.join_next().await;
                }

                let db = db.clone();
                let config_clone = config.clone();
                join_set.spawn(async move {
                    let _guard =
                        metrics::TokioTaskCounterGuard::new("startup_snapshot_accounts_cleanup");

                    db_queries::cleanup_accounts(
                        &db,
                        batch,
                        slot,
                        "snapshot_accounts",
                        Arc::new(Mutex::new(0)),
                        "cleanup_startup_snapshot_accounts_batch",
                        &config_clone,
                    )
                    .await;
                });
            }

            join_set.join_all().await;

            let elapsed = start_time.elapsed().as_secs_f64();
            tracing::info!(target: "cleanup_stored_accounts", "Cleaned up stored accounts from snapshot_accounts in {} seconds", elapsed);

            // Mark the service on DB as healthy
            db_queries::update_service_health(&db, true).await;
            *snapshot_processing_state
                .lock()
                .expect("Failed to lock snapshot_processing_state") =
                SnapshotProcessingState::FinishedAndCleanedUp;
        });
    }
}

///Used to buffer closure cleanups received while a gap fill is in progress, and replay them after
/// the gap fill finishes to catch accounts inserted by gap filling after the cleanup already ran.
#[derive(Clone, Default)]
pub struct PendingGapFillReplays {
    pub gap_fill_active: Arc<Mutex<bool>>,
    pub closures_buffer: Arc<Mutex<Vec<SlotAccounts>>>,
}

pub struct SlotAccounts {
    pub accounts: Vec<Vec<u8>>,
    pub slot: u64,
}

pub struct FinalizeSlotMessage {
    pub slot: u64,
    pub db: DatabaseConnection,
    pub updated_accounts: AccountsReceivedPerBlock,
    pub updated_accounts_during_startup: UpdatedAccountsDuringStartup,
    pub pending_gap_fill_replays: PendingGapFillReplays,
}

/// This serves the purpose of moving the finalize_slot logic to a background task to avoid blocking
/// the main thread, while keeping the execution of the finalize_slot function sequential to avoid db deadlocks.
pub fn start_finalize_slot_handler(
    config: &IndexConfig,
    finalize_slot_buffer_size: Arc<Mutex<usize>>,
) -> Sender<FinalizeSlotMessage> {
    let (tx, mut rx) =
        tokio::sync::mpsc::channel::<FinalizeSlotMessage>(config.finalize_slot_buffer_size);

    let config_clone = config.clone();
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            metrics::FINALIZE_SLOT_HANDLER_QUEUE_SIZE.set(rx.len() as i64);
            *finalize_slot_buffer_size
                .lock()
                .expect("Failed to lock finalize_slot_buffer_size") = rx.len();

            finalize_slot(
                &config_clone,
                msg.slot,
                msg.db,
                msg.updated_accounts,
                msg.updated_accounts_during_startup,
                msg.pending_gap_fill_replays,
            )
            .await;
        }
    });

    tx
}
