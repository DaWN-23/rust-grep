use std::path::PathBuf;

use dioxus::prelude::*;

use crate::export::{export_results, ExportFormat};
use crate::state::{AppState, SearchStatus};

#[component]
pub fn ToolBar() -> Element {
    let state = use_context::<AppState>();
    let search_dir = state.search_dir;
    let mut search_query = state.search_query;
    let status = state.status;
    let results = state.results;

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

    rsx! {
        div { class: "toolbar",
            // Path display
            span { class: "path-display", title: "{search_dir}", "{dir_display}" }

            // Folder picker button
            button {
                class: "btn",
                onclick: move |_| {
                    let mut dir = state.search_dir;
                    spawn(async move {
                        let folder = rfd::AsyncFileDialog::new().pick_folder().await;
                        if let Some(folder) = folder {
                            dir.set(folder.path().display().to_string());
                        }
                    });
                },
                "選択"
            }

            // Search input
            input {
                r#type: "text",
                placeholder: "検索文字列...",
                value: "{search_query}",
                oninput: move |e| {
                    search_query.set(e.value());
                },
                onkeydown: move |e: KeyboardEvent| {
                    if e.key() == Key::Enter {
                        state.engine.start(state);
                    }
                },
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
                        state.engine.start(state);
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
                        format!("エクスポート完了: {path_str}"),
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
                    status.set(SearchStatus::Error(format!("エクスポート失敗: {e}")));
                }
            }
        }
    });
}
