use std::ops::Range;

use regex::Regex;

use crate::state::SearchOptions;

/// Compiled matcher that handles both regex and literal search.
pub struct Matcher {
    regex: Regex,
}

impl Matcher {
    /// Build a Matcher from the query string and options.
    /// Returns `Err` if `use_regex` is true and the pattern is invalid.
    pub fn new(query: &str, options: &SearchOptions) -> Result<Self, regex::Error> {
        let pattern = if options.use_regex {
            query.to_string()
        } else {
            regex::escape(query)
        };

        let regex = if options.case_sensitive {
            Regex::new(&pattern)?
        } else {
            Regex::new(&format!("(?i){}", pattern))?
        };

        Ok(Self { regex })
    }

    /// Find all match ranges within a single line.
    pub fn find_matches(&self, line: &str) -> Vec<Range<usize>> {
        self.regex
            .find_iter(line)
            .map(|m| m.start()..m.end())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opts(use_regex: bool, case_sensitive: bool) -> SearchOptions {
        SearchOptions {
            use_regex,
            case_sensitive,
            ..Default::default()
        }
    }

    #[test]
    fn literal_case_insensitive() {
        let m = Matcher::new("hello", &opts(false, false)).unwrap();
        let ranges = m.find_matches("say Hello World");
        assert_eq!(ranges, vec![4..9]);
    }

    #[test]
    fn literal_case_sensitive() {
        let m = Matcher::new("hello", &opts(false, true)).unwrap();
        assert!(m.find_matches("say Hello World").is_empty());

        let ranges = m.find_matches("say hello world");
        assert_eq!(ranges, vec![4..9]);
    }

    #[test]
    fn literal_multiple_matches() {
        let m = Matcher::new("ab", &opts(false, false)).unwrap();
        let ranges = m.find_matches("ab cd ab");
        assert_eq!(ranges, vec![0..2, 6..8]);
    }

    #[test]
    fn literal_special_chars_escaped() {
        let m = Matcher::new("a.b", &opts(false, false)).unwrap();
        assert!(m.find_matches("acb").is_empty());
        assert_eq!(m.find_matches("a.b"), vec![0..3]);
    }

    #[test]
    fn regex_basic() {
        let m = Matcher::new(r"\d+", &opts(true, false)).unwrap();
        let ranges = m.find_matches("abc 123 def 456");
        assert_eq!(ranges, vec![4..7, 12..15]);
    }

    #[test]
    fn regex_case_sensitive() {
        let m = Matcher::new("Foo", &opts(true, true)).unwrap();
        assert!(m.find_matches("foo bar").is_empty());
        assert_eq!(m.find_matches("Foo bar"), vec![0..3]);
    }

    #[test]
    fn regex_invalid_pattern() {
        let result = Matcher::new("[invalid", &opts(true, false));
        assert!(result.is_err());
    }

    #[test]
    fn no_match_returns_empty() {
        let m = Matcher::new("xyz", &opts(false, false)).unwrap();
        assert!(m.find_matches("hello world").is_empty());
    }
}
