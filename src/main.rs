mod app;
mod export;
mod state;
mod components;
mod search;

fn main() {
    dioxus::launch(app::App);
}
