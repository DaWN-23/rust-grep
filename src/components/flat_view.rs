use dioxus::prelude::*;

use crate::state::AppState;
use super::highlight::HighlightedLine;

#[component]
pub fn FlatView() -> Element {
    let state = use_context::<AppState>();
    let results = state.results;

    rsx! {
        div { class: "flat-view",
            for (i, r) in results().iter().enumerate() {
                div { key: "{i}", class: "flat-match-line",
                    span { class: "flat-file-path", "{r.file_path.display()}" }
                    span { class: "flat-separator", ":{r.line_number}: " }
                    HighlightedLine {
                        text: r.line_content.clone(),
                        ranges: r.match_ranges.clone(),
                    }
                }
            }
        }
    }
}
