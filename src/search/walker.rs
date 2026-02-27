use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use content_inspector::inspect;
use ignore::WalkBuilder;

use crate::state::SearchOptions;

const BINARY_CHECK_SIZE: usize = 8192;

/// Check if a file is binary by inspecting the first 8KB.
pub fn is_binary(path: &Path) -> bool {
    let Ok(mut file) = File::open(path) else {
        return false;
    };
    let mut buf = [0u8; BINARY_CHECK_SIZE];
    let Ok(n) = file.read(&mut buf) else {
        return false;
    };
    inspect(&buf[..n]).is_binary()
}

/// Collect files to search from the given directory, respecting SearchOptions.
/// Uses `ignore::WalkBuilder` which respects .gitignore by default.
pub fn collect_files(dir: &Path, options: &SearchOptions) -> Vec<PathBuf> {
    let walker = WalkBuilder::new(dir).build();

    walker
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_some_and(|ft| ft.is_file()))
        .filter(|entry| {
            if options.max_file_size > 0 {
                entry
                    .metadata()
                    .ok()
                    .is_none_or(|m| m.len() <= options.max_file_size)
            } else {
                true
            }
        })
        .filter(|entry| {
            if !options.include_binary {
                !is_binary(entry.path())
            } else {
                true
            }
        })
        .map(|entry| entry.into_path())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    fn create_temp_dir() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    #[test]
    fn text_file_is_not_binary() {
        let dir = create_temp_dir();
        let file_path = dir.path().join("test.txt");
        fs::write(&file_path, "hello world\n").unwrap();
        assert!(!is_binary(&file_path));
    }

    #[test]
    fn binary_file_is_detected() {
        let dir = create_temp_dir();
        let file_path = dir.path().join("test.bin");
        let mut f = File::create(&file_path).unwrap();
        // Write bytes that include many null bytes — clearly binary
        let mut data = vec![0u8; 512];
        data[0] = 0x7f;
        data[1] = b'E';
        data[2] = b'L';
        data[3] = b'F';
        f.write_all(&data).unwrap();
        assert!(is_binary(&file_path));
    }

    #[test]
    fn collect_skips_binary_by_default() {
        let dir = create_temp_dir();

        // text file
        fs::write(dir.path().join("hello.txt"), "hello\n").unwrap();

        // binary file
        let bin_path = dir.path().join("data.bin");
        let mut f = File::create(&bin_path).unwrap();
        f.write_all(&vec![0u8; 512]).unwrap();

        let opts = SearchOptions {
            include_binary: false,
            ..Default::default()
        };
        let files = collect_files(dir.path(), &opts);
        assert!(files.iter().any(|p| p.ends_with("hello.txt")));
        assert!(!files.iter().any(|p| p.ends_with("data.bin")));
    }

    #[test]
    fn collect_includes_binary_when_option_set() {
        let dir = create_temp_dir();

        fs::write(dir.path().join("hello.txt"), "hello\n").unwrap();

        let bin_path = dir.path().join("data.bin");
        let mut f = File::create(&bin_path).unwrap();
        f.write_all(&vec![0u8; 512]).unwrap();

        let opts = SearchOptions {
            include_binary: true,
            ..Default::default()
        };
        let files = collect_files(dir.path(), &opts);
        assert!(files.iter().any(|p| p.ends_with("hello.txt")));
        assert!(files.iter().any(|p| p.ends_with("data.bin")));
    }

    #[test]
    fn collect_skips_large_files() {
        let dir = create_temp_dir();

        fs::write(dir.path().join("small.txt"), "small\n").unwrap();
        fs::write(dir.path().join("large.txt"), "x".repeat(2000)).unwrap();

        let opts = SearchOptions {
            max_file_size: 1000,
            ..Default::default()
        };
        let files = collect_files(dir.path(), &opts);
        assert!(files.iter().any(|p| p.ends_with("small.txt")));
        assert!(!files.iter().any(|p| p.ends_with("large.txt")));
    }
}
