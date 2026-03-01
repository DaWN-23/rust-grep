mod app;
mod editor;
mod export;
mod history;
mod settings;
mod state;
mod components;
mod search;

fn main() {
    env_logger::init();

    match history::ensure_data_dir() {
        Ok(dir) => log::info!("データディレクトリ: {}", dir.display()),
        Err(e) => log::error!("データディレクトリの初期化に失敗: {}", e),
    }

    dioxus::LaunchBuilder::new()
        .with_cfg(
            dioxus::desktop::Config::new()
                .with_window(
                    dioxus::desktop::WindowBuilder::new()
                        .with_title("Rust-Grep")
                        .with_inner_size(
                            dioxus::desktop::LogicalSize::new(1200.0, 800.0)
                        )
                )
        )
        .launch(app::App);
}
