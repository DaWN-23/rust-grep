use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::history::app_data_dir;
use crate::state::Theme;

const SETTINGS_FILE: &str = "settings.json";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppSettings {
    pub editor: EditorSettings,
    #[serde(default = "default_theme")]
    pub theme: Theme,
}

fn default_theme() -> Theme {
    Theme::System
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EditorSettings {
    /// コマンドテンプレート例:
    /// - VSCode  : "code --goto {file}:{line}"
    /// - Cursor  : "cursor --goto {file}:{line}"
    /// - Vim     : "vim +{line} {file}"
    /// - Neovim  : "nvim +{line} {file}"
    /// - 未設定  : "" （OS デフォルトで開く）
    pub command_template: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        let command_template = detect_default_editor().unwrap_or_default();
        Self {
            editor: EditorSettings { command_template },
            theme: Theme::System,
        }
    }
}

/// VSCode / Cursor が PATH に存在する場合はそのテンプレートを返す
fn detect_default_editor() -> Option<String> {
    // Check for common editors in order of preference
    let candidates = [
        ("code", "code --goto {file}:{line}"),
        ("cursor", "cursor --goto {file}:{line}"),
    ];

    for (cmd, template) in candidates {
        if command_exists(cmd) {
            return Some(template.to_string());
        }
    }
    None
}

fn command_exists(cmd: &str) -> bool {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("where")
            .arg(cmd)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::process::Command::new("which")
            .arg(cmd)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

/// Returns the path to the settings file inside the app data directory.
fn settings_path() -> PathBuf {
    app_data_dir().join(SETTINGS_FILE)
}

/// Load settings from disk. Returns default if file doesn't exist.
pub fn load_settings() -> AppSettings {
    let path = settings_path();

    match fs::read_to_string(&path) {
        Ok(data) => serde_json::from_str(&data).unwrap_or_else(|e| {
            log::warn!("設定の読み込みに失敗（デフォルト設定を使用）: {}", e);
            AppSettings::default()
        }),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => AppSettings::default(),
        Err(e) => {
            log::warn!("設定ファイルを開けません: {}", e);
            AppSettings::default()
        }
    }
}

/// Save settings to disk.
pub fn save_settings(settings: &AppSettings) {
    let path = settings_path();

    if let Some(parent) = path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            log::error!("設定ディレクトリの作成に失敗: {} ({})", parent.display(), e);
            return;
        }
    }

    let json = match serde_json::to_string_pretty(settings) {
        Ok(j) => j,
        Err(e) => {
            log::error!("設定のシリアライズに失敗: {}", e);
            return;
        }
    };

    if let Err(e) = fs::write(&path, json) {
        log::error!("設定の書き込みに失敗: {} ({})", path.display(), e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_settings_are_valid() {
        let settings = AppSettings::default();
        // command_template should be a string (empty or detected)
        assert!(
            settings.editor.command_template.is_empty()
                || settings.editor.command_template.contains("{file}")
        );
    }

    #[test]
    fn settings_roundtrip_json() {
        let settings = AppSettings {
            editor: EditorSettings {
                command_template: "code --goto {file}:{line}".to_string(),
            },
            theme: Theme::Dark,
        };
        let json = serde_json::to_string(&settings).unwrap();
        let parsed: AppSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(settings, parsed);
    }
}
