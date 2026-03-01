use std::path::PathBuf;

use dioxus::prelude::*;

use crate::export::{export_results, ExportFormat};
use crate::history::{remove_history_entry, remove_path_history_entry, save_path_history, save_query_history};
use crate::state::{AppState, SearchError, SearchStatus};

#[component]
pub fn ToolBar() -> Element {
    let mut state = use_context::<AppState>();
    let search_dir = state.search_dir;
    let mut search_query = state.search_query;
    let status = state.status;
    let results = state.results;
    let mut query_history = state.query_history;
    let mut path_history = state.path_history;

    let is_running = matches!(status(), SearchStatus::Running { .. });
    let is_done = matches!(status(), SearchStatus::Done { .. });
    let is_clearable = !search_query().is_empty()
        || !results().is_empty()
        || !matches!(status(), SearchStatus::Idle);
    let dir_display = if search_dir().is_empty() {
        "フォルダを選択...".to_string()
    } else {
        search_dir()
    };

    // Export dropdown open state
    let mut export_open = use_signal(|| false);
    // History dropdown open state
    let mut history_open = use_signal(|| false);
    // Path history dropdown open state
    let mut path_history_open = use_signal(|| false);

    // Helper: execute search and save to history
    let mut do_search = move || {
        let q = search_query();
        if !q.is_empty() {
            let mut h = query_history();
            if let Err(e) = save_query_history(&mut h, &q) {
                log::error!("クエリ履歴の保存に失敗: {}", e);
                state.history_error.set(Some(e));
            }
            query_history.set(h);
        }
        state.engine.start(state);
    };

    rsx! {
        div { class: "toolbar",
            // Path display + folder picker with path history
            div { class: "path-selector-wrapper",
                span { class: "path-display", title: "{search_dir}", "{dir_display}" }

                // Folder picker button
                button {
                    class: "btn",
                    onclick: move |_| {
                        let mut dir = state.search_dir;
                        spawn(async move {
                            let folder = rfd::AsyncFileDialog::new().pick_folder().await;
                            if let Some(folder) = folder {
                                let path = folder.path().to_path_buf();
                                dir.set(path.display().to_string());
                                // Save to path history
                                let mut h = path_history();
                                if let Err(e) = save_path_history(&mut h, &path) {
                                    log::error!("パス履歴の保存に失敗: {}", e);
                                    state.history_error.set(Some(e));
                                }
                                path_history.set(h);
                            }
                        });
                    },
                    "選択"
                }

                // Path history toggle button
                if !path_history().is_empty() {
                    button {
                        class: "btn",
                        onclick: move |_| {
                            path_history_open.set(!(path_history_open)());
                        },
                        "▼"
                    }
                }

                // Path history dropdown
                if (path_history_open)() && !path_history().is_empty() {
                    div { class: "history-dropdown path-history-dropdown",
                        for (i, item) in path_history().iter().enumerate() {
                            {
                                let item_str = item.display().to_string();
                                let item_str_click = item_str.clone();
                                let item_str_display = item_str.clone();
                                rsx! {
                                    div { class: "history-item",
                                        span {
                                            class: "history-item-text",
                                            title: "{item_str}",
                                            onclick: move |_| {
                                                let mut dir = state.search_dir;
                                                dir.set(item_str_click.clone());
                                                path_history_open.set(false);
                                            },
                                            "{item_str_display}"
                                        }
                                        button {
                                            class: "history-item-delete",
                                            onclick: move |_| {
                                                let mut h = path_history();
                                                if let Err(e) = remove_path_history_entry(&mut h, i) {
                                                    log::error!("パス履歴の削除に失敗: {}", e);
                                                }
                                                path_history.set(h);
                                            },
                                            "✕"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Search input with history dropdown
            div { class: "search-input-wrapper",
                input {
                    r#type: "text",
                    placeholder: "検索文字列...",
                    value: "{search_query}",
                    oninput: move |e| {
                        search_query.set(e.value());
                    },
                    onfocus: move |_| {
                        if !query_history().is_empty() {
                            history_open.set(true);
                        }
                    },
                    onblur: move |_| {
                        // Delay to allow click on dropdown items
                        spawn(async move {
                            tokio::time::sleep(std::time::Duration::from_millis(150)).await;
                            history_open.set(false);
                        });
                    },
                    onkeydown: move |e: KeyboardEvent| {
                        if e.key() == Key::Enter {
                            history_open.set(false);
                            do_search();
                        }
                    },
                }

                // History dropdown
                if (history_open)() && !query_history().is_empty() {
                    div { class: "history-dropdown",
                        for (i, item) in query_history().iter().enumerate() {
                            {
                                let item_clone = item.clone();
                                let item_display = item.clone();
                                rsx! {
                                    div { class: "history-item",
                                        span {
                                            class: "history-item-text",
                                            onclick: move |_| {
                                                search_query.set(item_clone.clone());
                                                history_open.set(false);
                                            },
                                            "{item_display}"
                                        }
                                        button {
                                            class: "history-item-delete",
                                            onclick: move |_| {
                                                let mut h = query_history();
                                                if let Err(e) = remove_history_entry(&mut h, i) {
                                                    log::error!("クエリ履歴の削除に失敗: {}", e);
                                                }
                                                query_history.set(h);
                                            },
                                            "✕"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Search / Cancel / Stop buttons
            if is_running {
                button {
                    class: "btn btn-danger",
                    onclick: move |_| {
                        state.engine.cancel();
                    },
                    "⏸ 中断"
                }
                button {
                    class: "btn btn-danger",
                    onclick: move |_| {
                        state.engine.stop(state);
                    },
                    "⏹ 中止"
                }
            } else {
                button {
                    class: "btn btn-primary",
                    disabled: search_query().is_empty() || search_dir().is_empty(),
                    onclick: move |_| {
                        do_search();
                    },
                    "検索"
                }
            }

            // Clear button (disabled during Running to prevent accidental use)
            button {
                class: "btn",
                disabled: !is_clearable || is_running,
                onclick: move |_| {
                    state.engine.clear(state);
                },
                "✕ クリア"
            }

            // Export dropdown
            div { class: "export-dropdown-wrapper",
                button {
                    class: "btn",
                    disabled: !is_done,
                    onclick: move |_| {
                        export_open.set(!(export_open)());
                    },
                    "エクスポート▼"
                }
                if (export_open)() && is_done {
                    div { class: "export-dropdown",
                        button {
                            class: "export-dropdown-item",
                            onclick: move |_| {
                                export_open.set(false);
                                do_export(state, results, ExportFormat::Csv);
                            },
                            "CSV で保存"
                        }
                        button {
                            class: "export-dropdown-item",
                            onclick: move |_| {
                                export_open.set(false);
                                do_export(state, results, ExportFormat::Tsv);
                            },
                            "TSV で保存"
                        }
                    }
                }
            }
        }
    }
}

fn do_export(
    state: AppState,
    results: Signal<Vec<crate::state::SearchResult>>,
    format: ExportFormat,
) {
    let mut status = state.status;

    spawn(async move {
        let ext = match format {
            ExportFormat::Csv => "csv",
            ExportFormat::Tsv => "tsv",
        };

        let dialog = rfd::AsyncFileDialog::new()
            .set_file_name(format!("search_results.{ext}"))
            .add_filter(ext.to_uppercase(), &[ext])
            .save_file()
            .await;

        if let Some(handle) = dialog {
            let path: PathBuf = handle.path().to_path_buf();
            let data = results();
            match export_results(&data, &path, format) {
                Ok(()) => {
                    let path_str = path.display().to_string();
                    status.set(SearchStatus::Done {
                        duration_ms: 0,
                        total_matches: data.len(),
                    });
                    // Temporarily show export success message
                    status.set(SearchStatus::Error(
                        SearchError::Unknown { message: format!("エクスポート完了: {path_str}") },
                    ));
                    // Revert to Done after 3 seconds
                    let matches = data.len();
                    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                    status.set(SearchStatus::Done {
                        duration_ms: 0,
                        total_matches: matches,
                    });
                }
                Err(e) => {
                    status.set(SearchStatus::Error(
                        SearchError::IoError { message: format!("エクスポート失敗: {e}") },
                    ));
                }
            }
        }
    });
}
