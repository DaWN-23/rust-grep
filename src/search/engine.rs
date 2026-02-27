use std::io::{BufRead, BufReader};
use std::fs::File;
use std::path::Path;
use std::time::Instant;

use rayon::prelude::*;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::state::{SearchOptions, SearchResult, SearchStatus};
use super::matcher::Matcher;
use super::walker::collect_files;

/// Message sent from search engine to the UI.
#[derive(Debug)]
pub enum SearchMessage {
    Result(SearchResult),
    Status(SearchStatus),
}

/// Run a search in the background.
/// Results are sent incrementally via the `tx` channel.
/// The search can be cancelled via the `cancel` token.
pub async fn run_search(
    dir: String,
    query: String,
    options: SearchOptions,
    tx: mpsc::Sender<SearchMessage>,
    cancel: CancellationToken,
) {
    let tx_err = tx.clone();
    let result = tokio::task::spawn_blocking(move || {
        run_search_blocking(&dir, &query, &options, &tx, &cancel)
    })
    .await;

    if let Err(e) = result {
        let _ = tx_err.send(SearchMessage::Status(SearchStatus::Error(e.to_string()))).await;
    }
}

fn run_search_blocking(
    dir: &str,
    query: &str,
    options: &SearchOptions,
    tx: &mpsc::Sender<SearchMessage>,
    cancel: &CancellationToken,
) {
    let start = Instant::now();

    // Build the matcher
    let matcher = match Matcher::new(query, options) {
        Ok(m) => m,
        Err(e) => {
            let _ = tx.blocking_send(SearchMessage::Status(SearchStatus::Error(
                format!("Invalid pattern: {}", e),
            )));
            return;
        }
    };

    let dir_path = Path::new(dir);
    if !dir_path.is_dir() {
        let _ = tx.blocking_send(SearchMessage::Status(SearchStatus::Error(
            format!("Not a directory: {}", dir),
        )));
        return;
    }

    // Collect files
    let files = collect_files(dir_path, options);

    if cancel.is_cancelled() {
        return;
    }

    // Send initial running status
    let _ = tx.blocking_send(SearchMessage::Status(SearchStatus::Running {
        scanned: 0,
        matched: 0,
    }));

    // Process files in parallel with rayon
    let results: Vec<Vec<SearchResult>> = files
        .par_iter()
        .filter_map(|file_path| {
            if cancel.is_cancelled() {
                return None;
            }
            Some(search_file(file_path, &matcher))
        })
        .collect();

    if cancel.is_cancelled() {
        return;
    }

    // Send results
    let mut total_matches = 0;
    for file_results in results {
        for result in file_results {
            total_matches += 1;
            if cancel.is_cancelled() {
                return;
            }
            let _ = tx.blocking_send(SearchMessage::Result(result));
        }
    }

    let duration_ms = start.elapsed().as_millis() as u64;
    let _ = tx.blocking_send(SearchMessage::Status(SearchStatus::Done {
        duration_ms,
        total_matches,
    }));
}

/// Search a single file for matching lines.
fn search_file(path: &Path, matcher: &Matcher) -> Vec<SearchResult> {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return Vec::new(),
    };

    let reader = BufReader::new(file);
    let mut results = Vec::new();

    for (line_idx, line) in reader.lines().enumerate() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };

        let ranges = matcher.find_matches(&line);
        if !ranges.is_empty() {
            results.push(SearchResult {
                file_path: path.to_path_buf(),
                line_number: line_idx + 1,
                line_content: line,
                match_ranges: ranges,
            });
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn create_temp_dir() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    #[tokio::test]
    async fn search_finds_literal_matches() {
        let dir = create_temp_dir();
        fs::write(dir.path().join("test.txt"), "hello world\nfoo bar\nhello again\n").unwrap();

        let (tx, mut rx) = mpsc::channel(100);
        let cancel = CancellationToken::new();

        run_search(
            dir.path().display().to_string(),
            "hello".to_string(),
            SearchOptions::default(),
            tx,
            cancel,
        )
        .await;

        let mut results = Vec::new();
        let mut done = false;
        while let Ok(msg) = rx.try_recv() {
            match msg {
                SearchMessage::Result(r) => results.push(r),
                SearchMessage::Status(SearchStatus::Done { total_matches, .. }) => {
                    assert_eq!(total_matches, 2);
                    done = true;
                }
                _ => {}
            }
        }
        assert!(done);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].line_number, 1);
        assert_eq!(results[1].line_number, 3);
    }

    #[tokio::test]
    async fn search_with_regex() {
        let dir = create_temp_dir();
        fs::write(dir.path().join("code.rs"), "let x = 42;\nlet y = abc;\nlet z = 100;\n").unwrap();

        let (tx, mut rx) = mpsc::channel(100);
        let cancel = CancellationToken::new();

        run_search(
            dir.path().display().to_string(),
            r"\d+".to_string(),
            SearchOptions {
                use_regex: true,
                ..Default::default()
            },
            tx,
            cancel,
        )
        .await;

        let mut results = Vec::new();
        while let Ok(msg) = rx.try_recv() {
            if let SearchMessage::Result(r) = msg {
                results.push(r);
            }
        }
        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn search_cancellation() {
        let dir = create_temp_dir();
        for i in 0..50 {
            fs::write(
                dir.path().join(format!("file_{}.txt", i)),
                "match_me\n".repeat(100),
            )
            .unwrap();
        }

        let (tx, _rx) = mpsc::channel(100);
        let cancel = CancellationToken::new();

        // Cancel immediately
        cancel.cancel();

        run_search(
            dir.path().display().to_string(),
            "match_me".to_string(),
            SearchOptions::default(),
            tx,
            cancel,
        )
        .await;

        // If we get here without hanging, cancellation works
    }

    #[tokio::test]
    async fn search_invalid_directory() {
        let (tx, mut rx) = mpsc::channel(100);
        let cancel = CancellationToken::new();

        run_search(
            "/nonexistent/path".to_string(),
            "test".to_string(),
            SearchOptions::default(),
            tx,
            cancel,
        )
        .await;

        let mut got_error = false;
        while let Ok(msg) = rx.try_recv() {
            if let SearchMessage::Status(SearchStatus::Error(_)) = msg {
                got_error = true;
            }
        }
        assert!(got_error);
    }

    #[tokio::test]
    async fn search_invalid_regex() {
        let dir = create_temp_dir();
        fs::write(dir.path().join("test.txt"), "hello\n").unwrap();

        let (tx, mut rx) = mpsc::channel(100);
        let cancel = CancellationToken::new();

        run_search(
            dir.path().display().to_string(),
            "[invalid".to_string(),
            SearchOptions {
                use_regex: true,
                ..Default::default()
            },
            tx,
            cancel,
        )
        .await;

        let mut got_error = false;
        while let Ok(msg) = rx.try_recv() {
            if let SearchMessage::Status(SearchStatus::Error(_)) = msg {
                got_error = true;
            }
        }
        assert!(got_error);
    }
}
