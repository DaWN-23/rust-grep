use std::fs::File;
use std::path::Path;

use crate::state::SearchResult;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExportFormat {
    Csv,
    Tsv,
}

pub fn export_results(
    results: &[SearchResult],
    path: &Path,
    format: ExportFormat,
) -> Result<(), Box<dyn std::error::Error>> {
    let delimiter = match format {
        ExportFormat::Csv => b',',
        ExportFormat::Tsv => b'\t',
    };

    let file = File::create(path)?;
    let mut wtr = csv::WriterBuilder::new()
        .delimiter(delimiter)
        .from_writer(file);

    // Header
    wtr.write_record(["file_path", "line_number", "content"])?;

    // Data rows
    for result in results {
        wtr.write_record([
            result.file_path.display().to_string(),
            result.line_number.to_string(),
            result.line_content.clone(),
        ])?;
    }

    wtr.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn sample_results() -> Vec<SearchResult> {
        vec![
            SearchResult {
                file_path: PathBuf::from("/src/main.rs"),
                line_number: 10,
                line_content: "fn main() {".to_string(),
                match_ranges: vec![3..7],
            },
            SearchResult {
                file_path: PathBuf::from("/src/lib.rs"),
                line_number: 42,
                line_content: "let x = \"hello, world\";".to_string(),
                match_ranges: vec![9..14],
            },
        ]
    }

    #[test]
    fn csv_export_correct_content() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("results.csv");

        export_results(&sample_results(), &path, ExportFormat::Csv).unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = content.lines().collect();

        assert_eq!(lines[0], "file_path,line_number,content");
        assert_eq!(lines[1], "/src/main.rs,10,fn main() {");
        // Second row has a comma in content — csv crate should quote it
        assert!(lines[2].contains("/src/lib.rs"));
        assert!(lines[2].contains("42"));
        assert!(lines[2].contains("hello, world"));
    }

    #[test]
    fn tsv_export_correct_content() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("results.tsv");

        export_results(&sample_results(), &path, ExportFormat::Tsv).unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = content.lines().collect();

        assert_eq!(lines[0], "file_path\tline_number\tcontent");
        assert_eq!(lines[1], "/src/main.rs\t10\tfn main() {");
        assert!(lines[2].contains("/src/lib.rs"));
        assert!(lines[2].contains("42"));
        assert!(lines[2].contains("hello, world"));
    }
}
