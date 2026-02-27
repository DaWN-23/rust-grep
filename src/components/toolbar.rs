use dioxus::prelude::*;

use crate::app::trigger_search;
use crate::state::{AppState, SearchStatus, SearchTrigger};

#[component]
pub fn ToolBar() -> Element {
    let state = use_context::<AppState>();
    let search_dir = state.search_dir;
    let mut search_query = state.search_query;
    let ui_settings = state.ui_settings;
    let status = state.status;

    let is_enter_mode = ui_settings().search_trigger == SearchTrigger::OnEnter;
    let is_running = matches!(status(), SearchStatus::Running { .. });
    let dir_display = if search_dir().is_empty() {
        "フォルダを選択...".to_string()
    } else {
        search_dir()
    };

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
                    if is_enter_mode && e.key() == Key::Enter {
                        trigger_search(state);
                    }
                },
            }

            // Search / Cancel button
            if is_running {
                button {
                    class: "btn btn-danger",
                    onclick: move |_| {
                        let cancel_token = state.cancel_token;
                        cancel_token().cancel();
                    },
                    "⏹ 中断"
                }
            } else {
                button {
                    class: "btn btn-primary",
                    disabled: !is_enter_mode || search_query().is_empty() || search_dir().is_empty(),
                    onclick: move |_| {
                        trigger_search(state);
                    },
                    "検索"
                }
            }
        }
    }
}
