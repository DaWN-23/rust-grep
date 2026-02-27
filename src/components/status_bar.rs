use dioxus::prelude::*;

use crate::state::{AppState, SearchStatus};

#[component]
pub fn StatusBar() -> Element {
    let state = use_context::<AppState>();
    let status = state.status;

    rsx! {
        div { class: "status-bar",
            match status() {
                SearchStatus::Idle => rsx! { span { "待機中" } },
                SearchStatus::Running { scanned, matched } => rsx! {
                    span { class: "running",
                        "検索中... {scanned}ファイルスキャン / {matched}件マッチ"
                    }
                },
                SearchStatus::Done { duration_ms, total_matches } => rsx! {
                    span { "{total_matches}件マッチ / {duration_ms}ms" }
                },
                SearchStatus::Cancelled { matched } => rsx! {
                    span { class: "cancelled",
                        "中断しました（{matched}件マッチ済み）"
                    }
                },
                SearchStatus::Error(msg) => rsx! {
                    span { class: "error", "エラー: {msg}" }
                },
            }
        }
    }
}
