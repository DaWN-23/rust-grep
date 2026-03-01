use std::ops::Range;
use std::path::PathBuf;

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use crate::history::{load_path_history, load_query_history};
use crate::search::search_engine::SearchEngine;
use crate::settings::{AppSettings, load_settings};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Theme {
    Dark,
    Light,
    System,
}

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

#[derive(Clone, PartialEq, Debug)]
pub enum SearchError {
    InvalidRegex {
        pattern: String,
        message: String,
    },
    PathNotFound {
        path: PathBuf,
    },
    PermissionDenied {
        path: PathBuf,
    },
    IoError {
        message: String,
    },
    Unknown {
        message: String,
    },
}

impl SearchError {
    pub fn title(&self) -> &str {
        match self {
            SearchError::InvalidRegex { .. } => "正規表現エラー",
            SearchError::PathNotFound { .. } => "パスが見つかりません",
            SearchError::PermissionDenied { .. } => "アクセス権限エラー",
            SearchError::IoError { .. } => "ディスク読み取りエラー",
            SearchError::Unknown { .. } => "予期しないエラー",
        }
    }

    pub fn description(&self) -> String {
        match self {
            SearchError::InvalidRegex { pattern, message } => {
                format!("パターン「{}」が不正です: {}", pattern, message)
            }
            SearchError::PathNotFound { path } => {
                format!("パス「{}」が存在しません", path.display())
            }
            SearchError::PermissionDenied { path } => {
                format!("パス「{}」へのアクセスが拒否されました", path.display())
            }
            SearchError::IoError { message } => {
                format!("I/Oエラー: {}", message)
            }
            SearchError::Unknown { message } => {
                format!("エラー: {}", message)
            }
        }
    }

    pub fn suggestion(&self) -> &str {
        match self {
            SearchError::InvalidRegex { .. } => "正規表現の構文を確認してください",
            SearchError::PathNotFound { .. } => "パスが存在するか確認してください",
            SearchError::PermissionDenied { .. } => "アクセス権限を確認してください",
            SearchError::IoError { .. } => "ディスクの状態を確認してください",
            SearchError::Unknown { .. } => "アプリを再起動してください",
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            SearchError::InvalidRegex { .. } => "\u{26a0}\u{fe0f}",
            SearchError::PathNotFound { .. } => "\u{1f4c1}",
            SearchError::PermissionDenied { .. } => "\u{1f512}",
            SearchError::IoError { .. } => "\u{1f4be}",
            SearchError::Unknown { .. } => "\u{2753}",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SearchStatus {
    Idle,
    Running { scanned: usize, matched: usize, elapsed_ms: u64, spinner_frame: usize },
    Done { duration_ms: u64, total_matches: usize },
    Cancelled { matched: usize },
    Error(SearchError),
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
    pub query_history: Signal<Vec<String>>,
    pub path_history: Signal<Vec<PathBuf>>,
    pub app_settings: Signal<AppSettings>,
    pub editor_error: Signal<Option<String>>,
    pub history_error: Signal<Option<String>>,
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
            query_history: Signal::new(load_query_history()),
            path_history: Signal::new(load_path_history()),
            app_settings: Signal::new(load_settings()),
            editor_error: Signal::new(None),
            history_error: Signal::new(None),
            engine: SearchEngine::new(),
        }
    }
}
