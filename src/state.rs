use std::ops::Range;
use std::path::PathBuf;

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use crate::search::search_engine::SearchEngine;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ResultViewMode {
    Tree,
    Flat,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiSettings {
    pub result_view_mode: ResultViewMode,
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            result_view_mode: ResultViewMode::Tree,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchOptions {
    pub use_regex: bool,
    pub case_sensitive: bool,
    pub include_binary: bool,
    pub max_file_size: u64,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            use_regex: false,
            case_sensitive: false,
            include_binary: false,
            max_file_size: 100 * 1024 * 1024, // 100MB
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SearchResult {
    pub file_path: PathBuf,
    pub line_number: usize,
    pub line_content: String,
    pub match_ranges: Vec<Range<usize>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SearchStatus {
    Idle,
    Running { scanned: usize, matched: usize, elapsed_ms: u64, spinner_frame: usize },
    Done { duration_ms: u64, total_matches: usize },
    Cancelled { matched: usize },
    Error(String),
}

/// Tracks which operation last transitioned to Idle,
/// so the StatusBar can show contextual messages.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LastAction {
    None,
    Stopped,
    Cleared,
}

#[derive(Clone, Copy)]
pub struct AppState {
    pub search_dir: Signal<String>,
    pub search_query: Signal<String>,
    pub ui_settings: Signal<UiSettings>,
    pub search_options: Signal<SearchOptions>,
    pub results: Signal<Vec<SearchResult>>,
    pub status: Signal<SearchStatus>,
    pub last_action: Signal<LastAction>,
    pub engine: SearchEngine,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            search_dir: Signal::new(String::new()),
            search_query: Signal::new(String::new()),
            ui_settings: Signal::new(UiSettings::default()),
            search_options: Signal::new(SearchOptions::default()),
            results: Signal::new(Vec::new()),
            status: Signal::new(SearchStatus::Idle),
            last_action: Signal::new(LastAction::None),
            engine: SearchEngine::new(),
        }
    }
}
