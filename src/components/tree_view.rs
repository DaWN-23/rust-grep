use std::collections::BTreeMap;

use dioxus::prelude::*;

use crate::state::{AppState, SearchResult};
use super::highlight::HighlightedLine;

#[component]
pub fn TreeView() -> Element {
    let state = use_context::<AppState>();
    let results = state.results;

    // Group results by file path (preserving insertion order via BTreeMap)
    let results_list = results();
    let mut grouped: BTreeMap<String, Vec<SearchResult>> = BTreeMap::new();
    for r in results_list {
        grouped
            .entry(r.file_path.display().to_string())
            .or_default()
            .push(r);
    }

    let entries: Vec<(String, Vec<SearchResult>)> = grouped.into_iter().collect();

    rsx! {
        div { class: "tree-view",
            for (path, matches) in entries {
                FileGroup { key: "{path}", path, matches }
            }
        }
    }
}

/// A single file group with fold/expand toggle.
#[component]
fn FileGroup(path: String, matches: Vec<SearchResult>) -> Element {
    let mut expanded = use_signal(|| true);
    let count = matches.len();

    rsx! {
        div { class: "tree-file-group",
            // File header (click to toggle)
            div {
                class: "tree-file-header",
                onclick: move |_| expanded.set(!expanded()),
                span { class: "tree-arrow",
                    if expanded() { "\u{25BE} " } else { "\u{25B8} " }
                }
                span { class: "tree-file-path", "{path}" }
                span { class: "tree-file-count", " ({count}件)" }
            }

            // Match lines
            if expanded() {
                div { class: "tree-file-lines",
                    for m in matches.iter() {
                        div {
                            key: "{m.line_number}",
                            class: "tree-match-line",
                            span { class: "line-number", "{m.line_number}:" }
                            HighlightedLine {
                                text: m.line_content.clone(),
                                ranges: m.match_ranges.clone(),
                            }
                        }
                    }
                }
            }
        }
    }
}
