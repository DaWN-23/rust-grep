use std::path::{Path, PathBuf};
use std::process::Command;

use crate::settings::EditorSettings;

/// Open a file in an external editor at the specified line.
///
/// - If `command_template` is empty, opens the file with the OS default app (no line number).
/// - Otherwise, parses the template into tokens, replaces `{file}` and `{line}`,
///   and spawns the command with proper argument separation (space-safe).
///
/// Runs on a blocking thread to avoid freezing the UI.
pub async fn open_in_editor(
    file: PathBuf,
    line: usize,
    settings: EditorSettings,
) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        open_in_editor_sync(&file, line, &settings)
    })
    .await
    .map_err(|e| format!("タスク実行エラー: {e}"))?
}

fn open_in_editor_sync(
    file: &Path,
    line: usize,
    settings: &EditorSettings,
) -> Result<(), String> {
    if settings.command_template.is_empty() {
        // Use OS default application (no line number)
        open::that(file.as_os_str()).map_err(|e| format!("ファイルを開けませんでした: {e}"))
    } else {
        let tokens = parse_command_template(&settings.command_template, file, line);
        if tokens.is_empty() {
            return Err("エディタコマンドが空です".to_string());
        }

        Command::new(&tokens[0])
            .args(&tokens[1..])
            .spawn()
            .map(|_| ())
            .map_err(|e| format!("エディタ起動失敗: {e}"))
    }
}

/// Parse a command template into a list of argument tokens.
///
/// - Replaces `{file}` with the file path and `{line}` with the line number.
/// - Handles `{file}:{line}` as a single combined token (e.g. for `code --goto {file}:{line}`).
/// - Respects quoted strings (single/double quotes) so spaces inside quotes are preserved.
/// - File paths with spaces are handled correctly because `{file}` replacement happens
///   before tokenization, and the replacement is treated as part of the current token.
fn parse_command_template(template: &str, file: &Path, line: usize) -> Vec<String> {
    let file_str = file.display().to_string();
    let line_str = line.to_string();

    // Replace placeholders first
    let expanded = template
        .replace("{file}", &file_str)
        .replace("{line}", &line_str);

    // Tokenize: split by whitespace but respect quotes
    shell_tokenize(&expanded)
}

/// Simple shell-like tokenizer that splits on whitespace but respects quotes.
fn shell_tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut chars = input.chars().peekable();
    let mut in_single_quote = false;
    let mut in_double_quote = false;

    while let Some(ch) = chars.next() {
        match ch {
            '\'' if !in_double_quote => {
                in_single_quote = !in_single_quote;
            }
            '"' if !in_single_quote => {
                in_double_quote = !in_double_quote;
            }
            ' ' | '\t' if !in_single_quote && !in_double_quote => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            _ => {
                current.push(ch);
            }
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_vscode_template() {
        let tokens = parse_command_template(
            "code --goto {file}:{line}",
            Path::new("/tmp/test.rs"),
            42,
        );
        assert_eq!(tokens, vec!["code", "--goto", "/tmp/test.rs:42"]);
    }

    #[test]
    fn parse_vim_template() {
        let tokens = parse_command_template(
            "vim +{line} {file}",
            Path::new("/tmp/test.rs"),
            10,
        );
        assert_eq!(tokens, vec!["vim", "+10", "/tmp/test.rs"]);
    }

    #[test]
    fn parse_path_with_spaces() {
        let tokens = parse_command_template(
            "code --goto '{file}:{line}'",
            Path::new("/Users/me/my project/file.rs"),
            5,
        );
        // Quoted: the entire argument (with spaces) stays as one token
        assert_eq!(tokens, vec!["code", "--goto", "/Users/me/my project/file.rs:5"]);
    }

    #[test]
    fn parse_neovim_template() {
        let tokens = parse_command_template(
            "nvim +{line} {file}",
            Path::new("/home/user/code/main.rs"),
            99,
        );
        assert_eq!(tokens, vec!["nvim", "+99", "/home/user/code/main.rs"]);
    }

    #[test]
    fn empty_template_returns_empty() {
        let tokens = parse_command_template("", Path::new("/tmp/test.rs"), 1);
        assert!(tokens.is_empty());
    }

    #[test]
    fn shell_tokenize_basic() {
        assert_eq!(shell_tokenize("a b c"), vec!["a", "b", "c"]);
    }

    #[test]
    fn shell_tokenize_quoted_spaces() {
        assert_eq!(
            shell_tokenize(r#"cmd "arg with spaces" other"#),
            vec!["cmd", "arg with spaces", "other"]
        );
    }

    #[test]
    fn shell_tokenize_single_quotes() {
        assert_eq!(
            shell_tokenize("cmd 'path with spaces' end"),
            vec!["cmd", "path with spaces", "end"]
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn parse_windows_backslash_path() {
        let tokens = parse_command_template(
            "code --goto {file}:{line}",
            Path::new(r"C:\Users\test\file.rs"),
            1,
        );
        assert_eq!(tokens, vec!["code", "--goto", r"C:\Users\test\file.rs:1"]);
    }
}
