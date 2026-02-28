use dioxus::prelude::*;

use crate::state::{AppState, LastAction, SearchStatus};

const SPINNER_FRAMES: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

#[component]
pub fn StatusBar() -> Element {
    let state = use_context::<AppState>();
    let status = state.status;
    let last_action = state.last_action;

    rsx! {
        div { class: "status-bar",
            match status() {
                SearchStatus::Idle => {
                    if last_action() == LastAction::Stopped {
                        rsx! { span { "中止しました。検索ボタンで再検索できます。" } }
                    } else {
                        rsx! { span { "待機中" } }
                    }
                },
                SearchStatus::Running { scanned, matched, elapsed_ms, spinner_frame } => {
                    if elapsed_ms >= 5000 {
                        let spinner = SPINNER_FRAMES[spinner_frame % SPINNER_FRAMES.len()];
                        let elapsed_sec = elapsed_ms / 1000;
                        rsx! {
                            span { class: "running",
                                "{spinner} 検索中です。しばらくお待ちください... ({elapsed_sec}s) / {matched}件マッチ / {scanned}ファイルスキャン済み"
                            }
                        }
                    } else {
                        rsx! {
                            span { class: "running",
                                "検索中... {matched}件マッチ / {scanned}ファイルスキャン済み ({elapsed_ms}ms)"
                            }
                        }
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
