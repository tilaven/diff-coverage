use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;

use super::CoverageSink;

#[derive(Default)]
pub struct CoverageStore {
    pub files: HashMap<String, FileCoverage>,
    normalized_ready: bool,
}

#[derive(Default, Clone, Debug)]
pub struct FileCoverage {
    pub measured_lines: Vec<u32>,
    pub covered_lines: Vec<u32>,
    dirty: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoverageLookupError {
    pub path: String,
    pub matches: Vec<String>,
}

impl fmt::Display for CoverageLookupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Ambiguous coverage match for '{}': {}",
            self.path,
            self.matches.join(", ")
        )
    }
}

impl std::error::Error for CoverageLookupError {}

impl CoverageStore {
    pub fn prepare_lookup(&mut self) {
        for coverage in self.files.values_mut() {
            coverage.normalize_in_place();
        }
        self.normalized_ready = true;
    }

    pub fn file_coverage<'a>(
        &'a self,
        path: &str,
    ) -> Result<Option<Cow<'a, FileCoverage>>, CoverageLookupError> {
        if let Some(coverage) = self.coverage_for(path) {
            return Ok(Some(coverage));
        }

        let mut matches = Vec::new();
        for key in self.files.keys() {
            if key.ends_with(path) || path.ends_with(key) {
                matches.push(key);
            }
        }

        match matches.len() {
            0 => Ok(None),
            1 => Ok(self.coverage_for(matches[0]).map(Some).unwrap_or(None)),
            _ => {
                let mut resolved = matches.into_iter().cloned().collect::<Vec<_>>();
                resolved.sort_unstable();
                Err(CoverageLookupError {
                    path: path.to_string(),
                    matches: resolved,
                })
            }
        }
    }

    fn coverage_for<'a>(&'a self, key: &str) -> Option<Cow<'a, FileCoverage>> {
        if self.normalized_ready {
            return self.files.get(key).map(Cow::Borrowed);
        }
        self.files
            .get(key)
            .map(|coverage| Cow::Owned(coverage.clone().normalized()))
    }
}

impl CoverageSink for CoverageStore {
    fn on_file(&mut self, file_path: &str) {
        self.normalized_ready = false;
        self.files.entry(file_path.to_string()).or_default();
    }

    fn on_line(&mut self, file_path: &str, line: u32, hits: u32) {
        self.normalized_ready = false;
        let entry = self.files.entry(file_path.to_string()).or_default();
        entry.record_line(line, hits);
    }
}

impl FileCoverage {
    pub fn record_line(&mut self, line: u32, hits: u32) {
        self.measured_lines.push(line);
        if hits > 0 {
            self.covered_lines.push(line);
        }
        self.dirty = true;
    }

    pub fn is_measured(&self, line: u32) -> bool {
        if self.dirty {
            return self.measured_lines.iter().any(|value| *value == line);
        }
        self.measured_lines.binary_search(&line).is_ok()
    }

    pub fn is_covered(&self, line: u32) -> bool {
        if self.dirty {
            return self.covered_lines.iter().any(|value| *value == line);
        }
        self.covered_lines.binary_search(&line).is_ok()
    }

    pub fn normalized(mut self) -> Self {
        self.normalize_in_place();
        self
    }

    fn normalize_in_place(&mut self) {
        if !self.dirty {
            return;
        }
        sort_and_dedup(&mut self.measured_lines);
        sort_and_dedup(&mut self.covered_lines);
        self.dirty = false;
    }
}

fn sort_and_dedup(values: &mut Vec<u32>) {
    values.sort_unstable();
    values.dedup();
}

#[cfg(test)]
mod tests {
    use super::{CoverageLookupError, CoverageStore, FileCoverage};
    use crate::coverage::CoverageSink;

    #[test]
    fn reports_ambiguous_path_matches() {
        let mut store = CoverageStore::default();
        store.on_line("src/foo.rs", 12, 0);
        store.on_line("src/foo.rs", 10, 2);
        store.on_line("lib/foo.rs", 14, 1);
        store.on_line("lib/foo.rs", 12, 1);

        let err = store
            .file_coverage("foo.rs")
            .expect_err("ambiguous coverage");
        assert_eq!(
            err,
            CoverageLookupError {
                path: "foo.rs".to_string(),
                matches: vec!["lib/foo.rs".to_string(), "src/foo.rs".to_string()],
            }
        );
    }

    #[test]
    fn prefers_exact_match_before_suffix_merge() {
        let mut store = CoverageStore::default();
        store.on_line("src/foo.rs", 12, 0);
        store.on_line("src/foo.rs", 10, 1);
        store.on_line("lib/foo.rs", 12, 1);

        let exact = store
            .file_coverage("src/foo.rs")
            .expect("lookup")
            .expect("exact coverage");
        assert_eq!(exact.measured_lines, vec![10, 12]);
        assert_eq!(exact.covered_lines, vec![10]);
    }

    #[test]
    fn normalize_sorts_and_dedups_lines() {
        let mut coverage = FileCoverage::default();
        coverage.record_line(4, 1);
        coverage.record_line(2, 1);
        coverage.record_line(4, 1);
        coverage.record_line(3, 0);

        let normalized = coverage.normalized();
        assert_eq!(normalized.measured_lines, vec![2, 3, 4]);
        assert_eq!(normalized.covered_lines, vec![2, 4]);
    }
}
