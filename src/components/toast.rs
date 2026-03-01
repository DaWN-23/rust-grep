use dioxus::prelude::*;

use crate::state::{AppState, SearchError, SearchStatus};

#[component]
pub fn SearchErrorToast() -> Element {
    let state = use_context::<AppState>();
    let status = state.status;
    let mut visible = use_signal(|| false);
    let mut last_error = use_signal(|| Option::<SearchError>::None);

    // Watch for new errors
    use_effect(move || {
        if let SearchStatus::Error(ref err) = status() {
            let err = err.clone();
            let is_unknown = matches!(err, SearchError::Unknown { .. });
            last_error.set(Some(err));
            visible.set(true);

            // Auto-dismiss after 5 seconds (except Unknown errors)
            if !is_unknown {
                spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                    visible.set(false);
                });
            }
        }
    });

    if !visible() {
        return rsx! {};
    }

    let Some(err) = last_error() else {
        return rsx! {};
    };

    let icon = err.icon();
    let title = err.title();

    rsx! {
        div { class: "search-toast",
            span { class: "search-toast-icon", "{icon}" }
            div { class: "search-toast-body",
                span { class: "search-toast-title", "{title}" }
                span { class: "search-toast-hint", "詳細はステータスバーを確認" }
            }
            button {
                class: "search-toast-close",
                onclick: move |_| {
                    visible.set(false);
                },
                "✕"
            }
        }
    }
}
