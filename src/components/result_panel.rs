use dioxus::prelude::*;

use crate::state::{AppState, ResultViewMode, SearchStatus};
use super::tree_view::TreeView;
use super::flat_view::FlatView;
use super::toast::SearchErrorToast;

#[component]
pub fn ResultPanel() -> Element {
    let state = use_context::<AppState>();
    let ui_settings = state.ui_settings;
    let results = state.results;
    let status = state.status;
    let editor_error = state.editor_error;

    let is_done_empty = matches!(
        status(),
        SearchStatus::Done { total_matches: 0, .. }
    );

    rsx! {
        div { class: "result-panel",
            if results().is_empty() {
                div { class: "result-empty",
                    if is_done_empty {
                        "マッチなし"
                    } else {
                        "検索結果がここに表示されます"
                    }
                }
            } else {
                match ui_settings().result_view_mode {
                    ResultViewMode::Tree => rsx! { TreeView {} },
                    ResultViewMode::Flat => rsx! { FlatView {} },
                }
            }

            // Editor error toast
            if let Some(err) = editor_error() {
                div { class: "editor-toast",
                    span { class: "editor-toast-text", "{err}" }
                    button {
                        class: "editor-toast-close",
                        onclick: move |_| {
                            let mut editor_error = state.editor_error;
                            editor_error.set(None);
                        },
                        "✕"
                    }
                }
            }

            // Search error toast
            SearchErrorToast {}
        }
    }
}
