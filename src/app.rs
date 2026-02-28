use dioxus::prelude::*;

use crate::components::options_bar::OptionsBar;
use crate::components::result_panel::ResultPanel;
use crate::components::status_bar::StatusBar;
use crate::components::toolbar::ToolBar;
use crate::state::AppState;

const MAIN_CSS: &str = include_str!("../assets/main.css");

#[component]
pub fn App() -> Element {
    use_context_provider(AppState::new);

    rsx! {
        style { {MAIN_CSS} }
        div { class: "app-container",
            ToolBar {}
            OptionsBar {}
            ResultPanel {}
            StatusBar {}
        }
    }
}
