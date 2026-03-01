use std::collections::{BTreeMap, HashSet};
use std::ops::Range;
use std::path::PathBuf;

use dioxus::prelude::*;

use crate::editor::open_in_editor;
use crate::state::{AppState, SearchResult};
use super::highlight::HighlightedLine;
use super::virtual_list::{ITEM_HEIGHT_PX, visible_range};

/// Flattened item for virtual scrolling in tree mode.
#[derive(Clone, PartialEq)]
enum TreeFlatItem {
    FileHeader {
        path: String,
        match_count: usize,
    },
    MatchLine {
        file_path: PathBuf,
        line_number: usize,
        line_content: String,
        match_ranges: Vec<Range<usize>>,
    },
}

#[component]
pub fn TreeView() -> Element {
    let state = use_context::<AppState>();
    let results = state.results;

    let mut scroll_top = use_signal(|| 0.0f64);
    let mut container_height = use_signal(|| 800.0f64);
    let mut collapsed = use_signal(HashSet::<String>::new);

    // Group results by file path
    let results_list = results();
    let mut grouped: BTreeMap<String, Vec<SearchResult>> = BTreeMap::new();
    for r in results_list {
        grouped
            .entry(r.file_path.display().to_string())
            .or_default()
            .push(r);
    }

    // Build flat items list based on collapse state
    let collapsed_set = collapsed();
    let mut flat_items: Vec<TreeFlatItem> = Vec::new();
    for (path, matches) in &grouped {
        flat_items.push(TreeFlatItem::FileHeader {
            path: path.clone(),
            match_count: matches.len(),
        });
        if !collapsed_set.contains(path) {
            for m in matches {
                flat_items.push(TreeFlatItem::MatchLine {
                    file_path: m.file_path.clone(),
                    line_number: m.line_number,
                    line_content: m.line_content.clone(),
                    match_ranges: m.match_ranges.clone(),
                });
            }
        }
    }

    let total = flat_items.len();
    let (start, end) = visible_range(scroll_top(), container_height(), total);
    let total_height = total as f64 * ITEM_HEIGHT_PX;
    let offset_top = start as f64 * ITEM_HEIGHT_PX;

    rsx! {
        div {
            class: "virtual-scroll-container",
            onmounted: move |e| {
                let data = e.data();
                spawn(async move {
                    if let Ok(rect) = data.get_client_rect().await {
                        container_height.set(rect.size.height);
                    }
                });
            },
            onscroll: move |e| {
                let data = e.data();
                scroll_top.set(data.scroll_top());
                container_height.set(data.client_height() as f64);
            },
            div {
                style: "height: {total_height}px; position: relative;",
                div {
                    style: "position: absolute; top: {offset_top}px; left: 0; right: 0;",
                    for i in start..end {
                        {
                            let item = &flat_items[i];
                            match item {
                                TreeFlatItem::FileHeader { path, match_count } => {
                                    let path_click = path.clone();
                                    let is_collapsed = collapsed_set.contains(path);
                                    rsx! {
                                        div {
                                            key: "h-{path}",
                                            class: "tree-file-header vscroll-item",
                                            onclick: move |_| {
                                                let mut c = collapsed();
                                                if c.contains(&path_click) {
                                                    c.remove(&path_click);
                                                } else {
                                                    c.insert(path_click.clone());
                                                }
                                                collapsed.set(c);
                                            },
                                            span { class: "tree-arrow",
                                                if is_collapsed { "\u{25B8} " } else { "\u{25BE} " }
                                            }
                                            span { class: "tree-file-path", "{path}" }
                                            span { class: "tree-file-count", " ({match_count}件)" }
                                        }
                                    }
                                }
                                TreeFlatItem::MatchLine { file_path, line_number, line_content, match_ranges } => {
                                    let fp = file_path.clone();
                                    let ln = *line_number;
                                    rsx! {
                                        div {
                                            key: "m-{i}",
                                            class: "tree-match-line clickable-row vscroll-item tree-match-indented",
                                            ondoubleclick: move |_| {
                                                let fp = fp.clone();
                                                async move {
                                                    let settings = (state.app_settings)();
                                                    if let Err(e) = open_in_editor(fp, ln, settings.editor.clone()).await {
                                                        let mut editor_error = state.editor_error;
                                                        editor_error.set(Some(e));
                                                        spawn(async move {
                                                            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                                                            editor_error.set(None);
                                                        });
                                                    }
                                                }
                                            },
                                            span { class: "line-number", "{line_number}:" }
                                            HighlightedLine {
                                                text: line_content.clone(),
                                                ranges: match_ranges.clone(),
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
