use std::ops::Range;

use dioxus::prelude::*;

/// Render a line with match ranges highlighted.
#[component]
pub fn HighlightedLine(text: String, ranges: Vec<Range<usize>>) -> Element {
    // Build segments: alternating between normal text and highlighted spans
    let mut segments: Vec<(String, bool)> = Vec::new();
    let mut cursor = 0;

    for range in &ranges {
        let start = range.start.min(text.len());
        let end = range.end.min(text.len());
        if start > cursor {
            segments.push((text[cursor..start].to_string(), false));
        }
        if start < end {
            segments.push((text[start..end].to_string(), true));
        }
        cursor = end;
    }
    if cursor < text.len() {
        segments.push((text[cursor..].to_string(), false));
    }

    rsx! {
        span { class: "hl-line",
            for (i, (seg, is_match)) in segments.into_iter().enumerate() {
                if is_match {
                    mark { key: "{i}", class: "hl-match", "{seg}" }
                } else {
                    span { key: "{i}", "{seg}" }
                }
            }
        }
    }
}
