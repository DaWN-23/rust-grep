use dioxus::prelude::*;
use dioxus::document::eval;

use crate::components::options_bar::OptionsBar;
use crate::components::result_panel::ResultPanel;
use crate::components::status_bar::StatusBar;
use crate::components::toolbar::ToolBar;
use crate::state::{AppState, Theme};

const MAIN_CSS: &str = include_str!("../assets/main.css");

#[component]
pub fn App() -> Element {
    use_context_provider(AppState::new);
    let state = use_context::<AppState>();
    let app_settings = state.app_settings;

    let mut os_prefers_dark = use_signal(|| true);

    // Detect OS theme preference on mount
    use_effect(move || {
        spawn(async move {
            let result = eval(
                r#"return window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches;"#
            ).await;
            if let Ok(val) = result {
                if let Some(dark) = val.as_bool() {
                    os_prefers_dark.set(dark);
                }
            }
        });
    });

    let theme_class = match app_settings().theme {
        Theme::Dark => "theme-dark",
        Theme::Light => "theme-light",
        Theme::System => {
            if os_prefers_dark() { "theme-dark" } else { "theme-light" }
        }
    };

    rsx! {
        style { {MAIN_CSS} }
        div { class: "app-container {theme_class}",
            ToolBar {}
            OptionsBar {}
            ResultPanel {}
            StatusBar {}
        }
    }
}
