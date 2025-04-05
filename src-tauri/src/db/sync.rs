//! Database synchronization module that provides a background synchronization service.
//! This module implements functionality for scheduling regular database syncs at specified intervals.

use crate::db::get_pool;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::time::{Duration, SystemTime};
use tauri::async_runtime::spawn;
use tokio::time;
use log::{debug, error, info, trace, warn};

/// Manages database synchronization on a regular interval.
///
/// This struct provides functionality to start and stop background synchronization
/// processes and track the timing of sync operations.
#[derive(Clone)]
pub struct SyncManager {
    /// Atomic flag indicating whether the sync service is running
    running: Arc<AtomicBool>,
    /// Time interval between synchronization operations
    interval: Duration,
    /// Timestamp of the last successful synchronization
    last_sync: Arc<Mutex<Option<SystemTime>>>,
    /// Scheduled timestamp for the next synchronization
    next_sync: Arc<Mutex<Option<SystemTime>>>,
}

impl SyncManager {
    /// Creates a new `SyncManager` with the specified interval.
    ///
    /// # Arguments
    ///
    /// * `interval` - The time interval in minutes between synchronization operations
    ///
    /// # Returns
    ///
    /// A new instance of `SyncManager`
    pub fn new(interval: u64) -> Self {
        info!("Creating new SyncManager with interval of {} minutes", interval);
        Self {
            running: Arc::new(AtomicBool::new(false)),
            interval: Duration::from_secs(interval * 60),
            last_sync: Arc::new(Mutex::new(None)),
            next_sync: Arc::new(Mutex::new(None)),
        }
    }

    /// Starts the synchronization service.
    ///
    /// This method launches a background task that performs an immediate sync,
    /// followed by regular sync operations at the configured interval.
    /// Timestamps for the last and next sync operations are updated after each successful sync.
    pub fn start(&self) {
        info!("Starting database synchronization service");
        let running = self.running.clone();

        if running.swap(true, Ordering::SeqCst) {
            warn!("Sync service already running, ignoring start request");
            return;
        }

        debug!("Synchronization interval set to {:?}", self.interval);
        let interval = self.interval;
        let last_sync = self.last_sync.clone();
        let next_sync = self.next_sync.clone();

        spawn(async move {
            trace!("Sync background task started");

            // Run sync immediately on startup
            if let Ok(Some(pool)) = get_pool().await {
                debug!("Running initial synchronization");
                match Self::sync(&pool).await {
                    Ok(_) => {
                        info!("Initial sync completed successfully");
                        // Update sync timing information
                        let now = SystemTime::now();
                        *last_sync.lock().unwrap() = Some(now);
                        let next = now + interval;
                        *next_sync.lock().unwrap() = Some(next);
                        debug!("Next sync scheduled for {:?}", next);
                    }
                    Err(e) => error!("Initial sync failed: {}", e),
                }
            } else {
                error!("Failed to acquire database pool for initial sync");
            }

            // Set up interval for future syncs
            debug!("Setting up periodic sync with interval {:?}", interval);
            let mut interval_timer = time::interval(interval);
            interval_timer.tick().await; // Consume first tick to avoid double-sync

            while running.load(Ordering::SeqCst) {
                trace!("Waiting for next sync interval");
                interval_timer.tick().await;
                trace!("Sync interval reached, starting synchronization");

                if let Ok(Some(pool)) = get_pool().await {
                    match Self::sync(&pool).await {
                        Ok(_) => {
                            info!("Background sync completed successfully");
                            // Update sync timing information
                            let now = SystemTime::now();
                            *last_sync.lock().unwrap() = Some(now);
                            let next = now + interval;
                            *next_sync.lock().unwrap() = Some(next);
                            debug!("Next sync scheduled for {:?}", next);
                        }
                        Err(e) => error!("Background sync failed: {}", e),
                    }
                } else {
                    error!("Failed to acquire database pool for scheduled sync");
                }
            }

            debug!("Sync background task terminated");
        });
    }

    /// Stops the synchronization service.
    ///
    /// This method signals the background task to terminate by setting the running flag to false.
    pub fn stop(&self) {
        info!("Stopping database synchronization service");
        let was_running = self.running.swap(false, Ordering::SeqCst);
        if !was_running {
            debug!("Sync service was not running");
        }
    }

    /// Calculates time remaining until the next scheduled sync.
    ///
    /// # Returns
    ///
    /// * `Some(Duration)` - The time remaining until the next sync
    /// * `None` - If no sync is scheduled or the next sync time is in the past
    pub fn time_until_next_sync(&self) -> Option<Duration> {
        trace!("Calculating time until next sync");
        let next_sync = self.next_sync.lock().unwrap();

        let result = next_sync.as_ref().and_then(|next| {
            let now = SystemTime::now();
            next.duration_since(now).ok()
        });

        match &result {
            Some(duration) => debug!("Time until next sync: {:?}", duration),
            None => debug!("No sync currently scheduled"),
        }

        result
    }

    /// Performs the actual synchronization work.
    ///
    /// # Arguments
    ///
    /// * `_` - SQLite connection pool
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If sync completed successfully
    /// * `Err(sqlx::Error)` - If sync failed
    ///
    /// Note: Currently this is a placeholder function with no actual implementation.
    async fn sync(_: &sqlx::SqlitePool) -> Result<(), sqlx::Error> {
        // Perform sync operations here
        Ok(())
    }
}
