use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use dioxus::prelude::*;
use tokio_util::sync::CancellationToken;

use crate::search::engine::{run_search, SearchMessage};
use crate::state::{AppState, LastAction, SearchOptions, SearchResult, SearchStatus};

/// Centralised search lifecycle manager.
///
/// Owns the cancellation token and exposes four operations:
///   - `start()`  — launch a new search (cancel any running one first)
///   - `cancel()` — interrupt: keep results, status → Cancelled
///   - `stop()`   — abort: clear results, status → Idle, keep query
///   - `clear()`  — reset all: clear results + query, status → Idle
///
/// Status updates (scanned/matched/elapsed/spinner) are driven by an
/// internal 50ms tick loop that reads atomic counters, independent of
/// the result channel.
#[derive(Clone, Copy)]
pub struct SearchEngine {
    cancel_token: Signal<CancellationToken>,
}

impl SearchEngine {
    pub fn new() -> Self {
        Self {
            cancel_token: Signal::new(CancellationToken::new()),
        }
    }

    /// Start a new search. Safe to call while another search is running —
    /// the previous search is cancelled first.
    ///
    /// Token lifecycle order (critical for cancel-button correctness):
    /// 1. old_token.cancel()           — stop any running search
    /// 2. new_token = new()            — create fresh token
    /// 3. self.cancel_token = new      — reflect BEFORE spawn
    /// 4. spawn execute_search         — start search LAST
    pub fn start(&self, state: AppState) {
        let dir = (state.search_dir)();
        let query = (state.search_query)();

        if dir.is_empty() || query.is_empty() {
            return;
        }

        // 1. Cancel previous search
        let mut cancel_token = self.cancel_token;
        cancel_token().cancel();

        // 2. Generate new token
        let token = CancellationToken::new();

        // 3. Reflect in state BEFORE spawning search
        cancel_token.set(token.clone());

        // Reset last_action since a new search is starting
        let mut last_action = state.last_action;
        last_action.set(LastAction::None);

        let options = (state.search_options)();
        let results = state.results;
        let status = state.status;

        // 4. Spawn search LAST
        spawn(async move {
            execute_search(dir, query, options, token, results, status).await;
        });
    }

    /// Cancel (interrupt) the currently running search.
    /// Results are preserved and status transitions to Cancelled.
    pub fn cancel(&self) {
        (self.cancel_token)().cancel();
    }

    /// Stop (abort) the running search: cancel the task, clear results,
    /// and reset status to Idle. Query is preserved for easy re-search.
    pub fn stop(&self, state: AppState) {
        self.cancel();

        let mut results = state.results;
        results.set(Vec::new());

        let mut status = state.status;
        status.set(SearchStatus::Idle);

        let mut last_action = state.last_action;
        last_action.set(LastAction::Stopped);
    }

    /// Clear all search state: cancel any running search, then reset
    /// results, status, and query to initial values.
    /// Search path is intentionally preserved.
    pub fn clear(&self, state: AppState) {
        self.cancel();

        let mut results = state.results;
        results.set(Vec::new());

        let mut status = state.status;
        status.set(SearchStatus::Idle);

        let mut query = state.search_query;
        query.set(String::new());

        let mut last_action = state.last_action;
        last_action.set(LastAction::Cleared);
    }
}

/// Core search execution with integrated 50ms tick loop for smooth status updates.
///
/// Architecture:
///   - Atomic counters (scanned/matched) are incremented lock-free by the search thread
///   - A 50ms tick loop reads counters and updates the status Signal independently
///   - The result channel only carries SearchResult and final status (Done/Error/Cancelled)
///   - Spinner frame advances every 100ms (every 2nd tick)
///   - Uses `tokio::select!` to interleave tick updates with channel reception
///     in a single async context (Dioxus Signal is not Send).
async fn execute_search(
    dir: String,
    query: String,
    options: SearchOptions,
    token: CancellationToken,
    mut results: Signal<Vec<SearchResult>>,
    mut status: Signal<SearchStatus>,
) {
    // Atomic counters shared with the blocking search thread
    let scanned_count = Arc::new(AtomicUsize::new(0));
    let matched_count = Arc::new(AtomicUsize::new(0));

    results.set(Vec::new());
    status.set(SearchStatus::Running {
        scanned: 0,
        matched: 0,
        elapsed_ms: 0,
        spinner_frame: 0,
    });

    let (tx, mut rx) = tokio::sync::mpsc::channel(1024);

    let handle = tokio::spawn(run_search(
        dir, query, options, tx, token,
        scanned_count.clone(), matched_count.clone(),
    ));

    // Interleave 50ms status ticks with channel message reception
    let start = Instant::now();
    let mut interval = tokio::time::interval(Duration::from_millis(50));
    interval.tick().await; // consume immediate first tick
    let mut tick_count: usize = 0;
    let mut channel_open = true;

    while channel_open {
        tokio::select! {
            biased;

            // Prioritise channel messages to avoid backpressure
            msg = rx.recv() => {
                match msg {
                    Some(SearchMessage::Result(r)) => {
                        results.write().push(r);
                    }
                    Some(SearchMessage::Status(s)) => {
                        // Final status (Done/Error/Cancelled)
                        status.set(s);
                        channel_open = false;
                    }
                    None => {
                        // Channel closed without final status
                        channel_open = false;
                    }
                }
            }

            // 50ms tick for smooth counter / spinner updates
            _ = interval.tick() => {
                tick_count += 1;
                let scanned = scanned_count.load(Ordering::Relaxed);
                let matched = matched_count.load(Ordering::Relaxed);
                let elapsed_ms = start.elapsed().as_millis() as u64;
                let spinner_frame = tick_count / 2;

                status.set(SearchStatus::Running {
                    scanned,
                    matched,
                    elapsed_ms,
                    spinner_frame,
                });
            }
        }
    }

    // Drain any remaining results after final status
    while let Ok(msg) = rx.try_recv() {
        if let SearchMessage::Result(r) = msg {
            results.write().push(r);
        }
    }

    drop(rx);
    let _ = handle.await;
}
