use dioxus::prelude::*;

use crate::state::{AppState, ResultViewMode, SearchTrigger};

#[component]
pub fn OptionsBar() -> Element {
    let state = use_context::<AppState>();
    let mut search_options = state.search_options;
    let mut ui_settings = state.ui_settings;

    let opts = search_options();
    let settings = ui_settings();

    rsx! {
        div { class: "options-bar",
            // Regex checkbox
            div { class: "option-group",
                label {
                    input {
                        r#type: "checkbox",
                        checked: opts.use_regex,
                        onchange: move |e| {
                            let mut o = search_options();
                            o.use_regex = e.checked();
                            search_options.set(o);
                        },
                    }
                    "正規表現"
                }
            }

            // Case sensitive checkbox
            div { class: "option-group",
                label {
                    input {
                        r#type: "checkbox",
                        checked: opts.case_sensitive,
                        onchange: move |e| {
                            let mut o = search_options();
                            o.case_sensitive = e.checked();
                            search_options.set(o);
                        },
                    }
                    "大文字小文字区別"
                }
            }

            // Include binary checkbox
            div { class: "option-group",
                label {
                    input {
                        r#type: "checkbox",
                        checked: opts.include_binary,
                        onchange: move |e| {
                            let mut o = search_options();
                            o.include_binary = e.checked();
                            search_options.set(o);
                        },
                    }
                    "バイナリ含む"
                }
            }

            // Separator
            div { style: "width: 1px; height: 16px; background: #45475a;" }

            // Tree / Flat toggle
            div { class: "toggle-group",
                button {
                    class: if settings.result_view_mode == ResultViewMode::Tree { "toggle-btn active" } else { "toggle-btn" },
                    onclick: move |_| {
                        let mut s = ui_settings();
                        s.result_view_mode = ResultViewMode::Tree;
                        ui_settings.set(s);
                    },
                    "Tree"
                }
                button {
                    class: if settings.result_view_mode == ResultViewMode::Flat { "toggle-btn active" } else { "toggle-btn" },
                    onclick: move |_| {
                        let mut s = ui_settings();
                        s.result_view_mode = ResultViewMode::Flat;
                        ui_settings.set(s);
                    },
                    "Flat"
                }
            }

            // Enter / Live toggle
            div { class: "toggle-group",
                button {
                    class: if settings.search_trigger == SearchTrigger::OnEnter { "toggle-btn active" } else { "toggle-btn" },
                    onclick: move |_| {
                        let mut s = ui_settings();
                        s.search_trigger = SearchTrigger::OnEnter;
                        ui_settings.set(s);
                    },
                    "Enter"
                }
                button {
                    class: if settings.search_trigger == SearchTrigger::Realtime { "toggle-btn active" } else { "toggle-btn" },
                    onclick: move |_| {
                        let mut s = ui_settings();
                        s.search_trigger = SearchTrigger::Realtime;
                        ui_settings.set(s);
                    },
                    "Live"
                }
            }
        }
    }
}
