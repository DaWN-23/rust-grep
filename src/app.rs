use std::time::Duration;

use dioxus::prelude::*;
use tokio_util::sync::CancellationToken;

use crate::components::options_bar::OptionsBar;
use crate::components::result_panel::ResultPanel;
use crate::components::status_bar::StatusBar;
use crate::components::toolbar::ToolBar;
use crate::search::engine::{run_search, SearchMessage};
use crate::state::{AppState, SearchOptions, SearchResult, SearchStatus, SearchTrigger};

const MAIN_CSS: &str = include_str!("../assets/main.css");

#[component]
pub fn App() -> Element {
    use_context_provider(AppState::new);
    let state = use_context::<AppState>();

    // Realtime mode: debounce + auto-search
    use_effect(move || {
        let query = (state.search_query)();
        let dir = (state.search_dir)();
        let settings = (state.ui_settings)();
        let options = (state.search_options)();

        // Only active in Realtime mode
        if settings.search_trigger != SearchTrigger::Realtime {
            return;
        }

        // Cancel any previous debounce/search
        let mut cancel_token = state.cancel_token;
        cancel_token().cancel();

        // Empty query → reset to idle
        if query.is_empty() || dir.is_empty() {
            let mut results = state.results;
            let mut status = state.status;
            results.set(Vec::new());
            status.set(SearchStatus::Idle);
            return;
        }

        // New cancellation token for this debounce + search
        let token = CancellationToken::new();
        cancel_token.set(token.clone());

        let debounce_ms = settings.debounce_ms;
        let results = state.results;
        let status = state.status;

        spawn(async move {
            // Debounce wait
            let check = token.clone();
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_millis(debounce_ms)) => {},
                _ = check.cancelled() => { return; }
            }

            if token.is_cancelled() {
                return;
            }

            execute_search(dir, query, options, token, results, status).await;
        });
    });

    rsx! {
        style { {MAIN_CSS} }
        div { class: "app-container",
            ToolBar {}
            OptionsBar {}
            ResultPanel {}
            StatusBar {}
        }
    }
}

/// Start a search immediately (called from OnEnter mode).
pub fn trigger_search(state: AppState) {
    let dir = (state.search_dir)();
    let query = (state.search_query)();

    if dir.is_empty() || query.is_empty() {
        return;
    }

    // Cancel previous search
    let mut cancel_token = state.cancel_token;
    cancel_token().cancel();
    let token = CancellationToken::new();
    cancel_token.set(token.clone());

    let options = (state.search_options)();
    let results = state.results;
    let status = state.status;

    spawn(async move {
        execute_search(dir, query, options, token, results, status).await;
    });
}

/// Core search execution: clears results, runs engine, receives messages.
async fn execute_search(
    dir: String,
    query: String,
    options: SearchOptions,
    token: CancellationToken,
    mut results: Signal<Vec<SearchResult>>,
    mut status: Signal<SearchStatus>,
) {
    results.set(Vec::new());
    status.set(SearchStatus::Running {
        scanned: 0,
        matched: 0,
    });

    let (tx, mut rx) = tokio::sync::mpsc::channel(1024);
    tokio::spawn(run_search(dir, query, options, tx, token));

    while let Some(msg) = rx.recv().await {
        match msg {
            SearchMessage::Result(r) => {
                results.write().push(r);
            }
            SearchMessage::Status(s) => {
                status.set(s);
            }
        }
    }
}
