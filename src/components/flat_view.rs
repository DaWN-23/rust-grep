use dioxus::prelude::*;

use crate::editor::open_in_editor;
use crate::state::AppState;
use super::highlight::HighlightedLine;
use super::virtual_list::{ITEM_HEIGHT_PX, visible_range};

#[component]
pub fn FlatView() -> Element {
    let state = use_context::<AppState>();
    let results = state.results;

    let mut scroll_top = use_signal(|| 0.0f64);
    let mut container_height = use_signal(|| 800.0f64);

    let items = results();
    let total = items.len();
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
            // Total height spacer
            div {
                style: "height: {total_height}px; position: relative;",
                // Visible items positioned at offset
                div {
                    style: "position: absolute; top: {offset_top}px; left: 0; right: 0;",
                    for i in start..end {
                        {
                            let r = &items[i];
                            let file_path = r.file_path.clone();
                            let line_number = r.line_number;
                            rsx! {
                                div {
                                    key: "{i}",
                                    class: "flat-match-line clickable-row vscroll-item",
                                    ondoubleclick: move |_| {
                                        let file_path = file_path.clone();
                                        async move {
                                            let settings = (state.app_settings)();
                                            if let Err(e) = open_in_editor(file_path, line_number, settings.editor.clone()).await {
                                                let mut editor_error = state.editor_error;
                                                editor_error.set(Some(e));
                                                spawn(async move {
                                                    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                                                    editor_error.set(None);
                                                });
                                            }
                                        }
                                    },
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
            }
        }
    }
}
