use std::fs;
use std::path::{Path, PathBuf};

const MAX_HISTORY: usize = 50;
const QUERY_HISTORY_FILE: &str = "query_history.json";

const MAX_PATH_HISTORY: usize = 20;
const PATH_HISTORY_FILE: &str = "path_history.json";

/// Returns the application data directory, resolved per-platform.
///
/// Priority:
/// 1. `RUST_GREP_DATA_DIR` env var (for tests / dev override)
/// 2. `dirs::data_local_dir()` / "rust-grep"
///    - macOS: ~/Library/Application Support/rust-grep/
///    - Windows: C:\Users\<user>\AppData\Local\rust-grep\
/// 3. Fallback: `dirs::home_dir()` / ".rust-grep"
/// 4. Final fallback: "./.rust-grep" (current directory)
pub fn app_data_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("RUST_GREP_DATA_DIR") {
        return PathBuf::from(dir);
    }

    if let Some(dir) = dirs::data_local_dir() {
        return dir.join("rust-grep");
    }

    if let Some(home) = dirs::home_dir() {
        return home.join(".rust-grep");
    }

    PathBuf::from(".rust-grep")
}

/// Ensure the data directory exists, creating it if necessary.
/// Call this at app startup.
pub fn ensure_data_dir() -> Result<PathBuf, String> {
    let dir = app_data_dir();
    fs::create_dir_all(&dir)
        .map_err(|e| format!("データディレクトリの作成に失敗: {} ({})", dir.display(), e))?;
    Ok(dir)
}

// ── Query History ──

/// Load query history from disk. Returns empty Vec on any failure.
pub fn load_query_history() -> Vec<String> {
    let path = app_data_dir().join(QUERY_HISTORY_FILE);
    match fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_else(|e| {
            log::warn!("クエリ履歴の読み込みに失敗（空の履歴を使用）: {}", e);
            Vec::new()
        }),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(e) => {
            log::warn!("クエリ履歴ファイルを開けません: {}", e);
            Vec::new()
        }
    }
}

/// Add a query to the history and persist to disk.
///
/// - Empty strings are ignored.
/// - Duplicate queries are moved to the front.
/// - Excess entries beyond MAX_HISTORY are trimmed from the end.
pub fn save_query_history(history: &mut Vec<String>, query: &str) -> Result<(), String> {
    if query.is_empty() {
        return Ok(());
    }

    // Remove existing duplicate
    history.retain(|q| q != query);

    // Insert at front
    history.insert(0, query.to_string());

    // Trim to max
    history.truncate(MAX_HISTORY);

    // Persist
    persist_query_history(history)
}

/// Remove a single entry from history and persist.
pub fn remove_history_entry(history: &mut Vec<String>, index: usize) -> Result<(), String> {
    if index < history.len() {
        history.remove(index);
        persist_query_history(history)?;
    }
    Ok(())
}

fn persist_query_history(history: &[String]) -> Result<(), String> {
    let path = app_data_dir().join(QUERY_HISTORY_FILE);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("ディレクトリ作成失敗: {} ({})", parent.display(), e))?;
    }

    let json = serde_json::to_string_pretty(history)
        .map_err(|e| format!("クエリ履歴のシリアライズに失敗: {}", e))?;

    fs::write(&path, json)
        .map_err(|e| format!("クエリ履歴の書き込みに失敗: {} ({})", path.display(), e))?;

    Ok(())
}

// ── Path History ──

/// Load path history from disk. Returns empty Vec on any failure.
pub fn load_path_history() -> Vec<PathBuf> {
    let path = app_data_dir().join(PATH_HISTORY_FILE);
    match fs::read_to_string(&path) {
        Ok(content) => {
            let strings: Vec<String> = serde_json::from_str(&content).unwrap_or_else(|e| {
                log::warn!("パス履歴の読み込みに失敗（空の履歴を使用）: {}", e);
                Vec::new()
            });
            strings.into_iter().map(PathBuf::from).collect()
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(e) => {
            log::warn!("パス履歴ファイルを開けません: {}", e);
            Vec::new()
        }
    }
}

/// Add a path to the history and persist to disk.
///
/// - Duplicate paths are moved to the front.
/// - Excess entries beyond MAX_PATH_HISTORY are trimmed from the end.
pub fn save_path_history(history: &mut Vec<PathBuf>, path: &Path) -> Result<(), String> {
    let path_buf = path.to_path_buf();

    // Remove existing duplicate
    history.retain(|p| p != &path_buf);

    // Insert at front
    history.insert(0, path_buf);

    // Trim to max
    history.truncate(MAX_PATH_HISTORY);

    // Persist
    persist_path_history(history)
}

/// Remove a single entry from path history and persist.
pub fn remove_path_history_entry(history: &mut Vec<PathBuf>, index: usize) -> Result<(), String> {
    if index < history.len() {
        history.remove(index);
        persist_path_history(history)?;
    }
    Ok(())
}

fn persist_path_history(history: &[PathBuf]) -> Result<(), String> {
    let path = app_data_dir().join(PATH_HISTORY_FILE);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("ディレクトリ作成失敗: {} ({})", parent.display(), e))?;
    }

    let strings: Vec<String> = history.iter().map(|p| p.display().to_string()).collect();
    let json = serde_json::to_string_pretty(&strings)
        .map_err(|e| format!("パス履歴のシリアライズに失敗: {}", e))?;

    fs::write(&path, json)
        .map_err(|e| format!("パス履歴の書き込みに失敗: {} ({})", path.display(), e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Env vars are process-global — serialize all history tests that use them.
    static TEST_LOCK: Mutex<()> = Mutex::new(());

    fn setup_temp_dir() -> (tempfile::TempDir, std::sync::MutexGuard<'static, ()>) {
        let guard = TEST_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let data_subdir = dir.path().join("data");
        unsafe {
            std::env::set_var("RUST_GREP_DATA_DIR", &data_subdir);
        }
        (dir, guard)
    }

    #[test]
    fn save_adds_to_front() {
        let (_dir, _lock) = setup_temp_dir();
        ensure_data_dir().unwrap();
        let mut history = vec!["old".to_string()];
        save_query_history(&mut history, "new").unwrap();
        assert_eq!(history, vec!["new", "old"]);
    }

    #[test]
    fn save_deduplicates() {
        let (_dir, _lock) = setup_temp_dir();
        ensure_data_dir().unwrap();
        let mut history = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        save_query_history(&mut history, "b").unwrap();
        assert_eq!(history, vec!["b", "a", "c"]);
    }

    #[test]
    fn save_ignores_empty() {
        let (_dir, _lock) = setup_temp_dir();
        let mut history = vec!["a".to_string()];
        save_query_history(&mut history, "").unwrap();
        assert_eq!(history, vec!["a"]);
    }

    #[test]
    fn save_truncates_to_max() {
        let (_dir, _lock) = setup_temp_dir();
        ensure_data_dir().unwrap();
        let mut history: Vec<String> = (0..MAX_HISTORY).map(|i| format!("q{i}")).collect();
        save_query_history(&mut history, "new").unwrap();
        assert_eq!(history.len(), MAX_HISTORY);
        assert_eq!(history[0], "new");
    }

    #[test]
    fn remove_entry() {
        let (_dir, _lock) = setup_temp_dir();
        ensure_data_dir().unwrap();
        let mut history = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        remove_history_entry(&mut history, 1).unwrap();
        assert_eq!(history, vec!["a", "c"]);
    }

    #[test]
    fn path_save_adds_to_front() {
        let (_dir, _lock) = setup_temp_dir();
        ensure_data_dir().unwrap();
        let mut history = vec![PathBuf::from("/old")];
        save_path_history(&mut history, Path::new("/new")).unwrap();
        assert_eq!(history, vec![PathBuf::from("/new"), PathBuf::from("/old")]);
    }

    #[test]
    fn path_save_deduplicates() {
        let (_dir, _lock) = setup_temp_dir();
        ensure_data_dir().unwrap();
        let mut history = vec![PathBuf::from("/a"), PathBuf::from("/b"), PathBuf::from("/c")];
        save_path_history(&mut history, Path::new("/b")).unwrap();
        assert_eq!(history, vec![PathBuf::from("/b"), PathBuf::from("/a"), PathBuf::from("/c")]);
    }

    #[test]
    fn path_save_truncates_to_max() {
        let (_dir, _lock) = setup_temp_dir();
        ensure_data_dir().unwrap();
        let mut history: Vec<PathBuf> = (0..MAX_PATH_HISTORY).map(|i| PathBuf::from(format!("/p{i}"))).collect();
        save_path_history(&mut history, Path::new("/new")).unwrap();
        assert_eq!(history.len(), MAX_PATH_HISTORY);
        assert_eq!(history[0], PathBuf::from("/new"));
    }

    #[test]
    fn path_remove_entry() {
        let (_dir, _lock) = setup_temp_dir();
        ensure_data_dir().unwrap();
        let mut history = vec![PathBuf::from("/a"), PathBuf::from("/b"), PathBuf::from("/c")];
        remove_path_history_entry(&mut history, 1).unwrap();
        assert_eq!(history, vec![PathBuf::from("/a"), PathBuf::from("/c")]);
    }

    // ── Integration tests ──

    #[test]
    fn test_query_history_save_and_load() {
        let (_dir, _lock) = setup_temp_dir();
        ensure_data_dir().unwrap();

        let mut history = Vec::new();
        save_query_history(&mut history, "hello").unwrap();
        save_query_history(&mut history, "world").unwrap();

        let loaded = load_query_history();
        assert_eq!(loaded, vec!["world", "hello"]);
    }

    #[test]
    fn test_path_history_save_and_load() {
        let (_dir, _lock) = setup_temp_dir();
        ensure_data_dir().unwrap();

        let mut history = Vec::new();
        save_path_history(&mut history, Path::new("/usr/local")).unwrap();
        save_path_history(&mut history, Path::new("/tmp")).unwrap();

        let loaded = load_path_history();
        assert_eq!(loaded, vec![PathBuf::from("/tmp"), PathBuf::from("/usr/local")]);
    }

    #[test]
    fn test_load_returns_empty_when_no_file() {
        let (_dir, _lock) = setup_temp_dir();
        ensure_data_dir().unwrap();

        let queries = load_query_history();
        assert!(queries.is_empty());

        let paths = load_path_history();
        assert!(paths.is_empty());
    }

    #[test]
    fn test_ensure_data_dir_creates_directory() {
        let (_dir, _lock) = setup_temp_dir();
        let data_dir = app_data_dir();

        // "data" subdirectory should not exist yet
        assert!(!data_dir.exists());

        ensure_data_dir().unwrap();

        // Now it should exist
        assert!(data_dir.is_dir());
    }
}
