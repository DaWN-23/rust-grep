use dioxus::prelude::*;

use crate::settings::save_settings;
use crate::state::{AppState, ResultViewMode, Theme};

#[component]
pub fn OptionsBar() -> Element {
    let mut state = use_context::<AppState>();
    let mut search_options = state.search_options;
    let mut ui_settings = state.ui_settings;

    let opts = search_options();
    let settings = ui_settings();

    let mut editor_open = use_signal(|| false);

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
            div { class: "separator" }

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

            // Separator
            div { class: "separator" }

            // Theme toggle
            button {
                class: "btn",
                onclick: move |_| {
                    let mut s = (state.app_settings)();
                    s.theme = match s.theme {
                        Theme::Dark => Theme::Light,
                        Theme::Light => Theme::System,
                        Theme::System => Theme::Dark,
                    };
                    save_settings(&s);
                    state.app_settings.set(s);
                },
                {match (state.app_settings)().theme {
                    Theme::Dark => "Dark",
                    Theme::Light => "Light",
                    Theme::System => "System",
                }}
            }

            // Separator
            div { class: "separator" }

            // Editor settings toggle
            button {
                class: if (editor_open)() { "btn btn-primary" } else { "btn" },
                onclick: move |_| {
                    editor_open.set(!(editor_open)());
                },
                "エディタ設定"
            }
        }

        // Editor settings panel (collapsible)
        if (editor_open)() {
            EditorSettingsPanel {}
        }
    }
}

#[component]
fn EditorSettingsPanel() -> Element {
    let state = use_context::<AppState>();
    let mut app_settings = state.app_settings;
    let current_template = (app_settings)().editor.command_template.clone();
    let mut input_value = use_signal(|| current_template);

    let presets = [
        ("VSCode", "code --goto {file}:{line}"),
        ("Cursor", "cursor --goto {file}:{line}"),
        ("Vim", "vim +{line} {file}"),
        ("Neovim", "nvim +{line} {file}"),
    ];

    rsx! {
        div { class: "editor-settings-panel",
            div { class: "editor-settings-row",
                span { class: "editor-settings-label", "コマンドテンプレート:" }
                input {
                    r#type: "text",
                    class: "editor-settings-input",
                    placeholder: "例: code --goto {{file}}:{{line}}",
                    value: "{input_value}",
                    oninput: move |e| {
                        input_value.set(e.value());
                    },
                }
                button {
                    class: "btn btn-primary",
                    onclick: move |_| {
                        let mut s = (app_settings)();
                        s.editor.command_template = (input_value)();
                        save_settings(&s);
                        app_settings.set(s);
                    },
                    "保存"
                }
            }
            div { class: "editor-presets-row",
                span { class: "editor-settings-label", "プリセット:" }
                for (name, template) in presets {
                    button {
                        class: "btn editor-preset-btn",
                        onclick: move |_| {
                            input_value.set(template.to_string());
                        },
                        "{name}"
                    }
                }
                button {
                    class: "btn editor-preset-btn",
                    onclick: move |_| {
                        input_value.set(String::new());
                    },
                    "クリア"
                }
            }
            div { class: "editor-settings-hint",
                "空欄の場合はOSデフォルトアプリで開きます。プレースホルダー: {{file}} {{line}}"
            }
        }
    }
}
