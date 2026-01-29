pub mod clover;
pub mod cobertura;
pub mod store;

use std::io::Read;

use std::collections::BTreeSet;

use crate::diff::types::ChangedFile;
use crate::report::{CoverageReport, UncoveredFile};
use crate::util::path::normalize_path;
use store::CoverageStore;

pub trait CoverageParser {
    fn can_parse<R: Read>(&self, reader: R) -> std::io::Result<bool>;
    fn parse<R: Read>(&self, reader: R, sink: &mut dyn CoverageSink) -> std::io::Result<()>;
}

pub trait CoverageSink {
    fn on_file(&mut self, file_path: &str);
    fn on_line(&mut self, file_path: &str, line: u32, hits: u32);
}

pub fn analyze_changed_coverage(
    changed_files: &[ChangedFile],
    coverage: &CoverageStore,
    treat_missing_as_uncovered: bool,
) -> Result<CoverageReport, store::CoverageLookupError> {
    let mut uncovered_files = Vec::new();
    let mut total_changed = 0usize;
    let mut total_covered = 0usize;

    for changed_file in changed_files {
        let normalized_path = normalize_path(&changed_file.path);
        let unique_lines: BTreeSet<u32> = changed_file.changed_lines.iter().copied().collect();
        if unique_lines.is_empty() {
            continue;
        }

        let file_coverage = coverage.file_coverage(&normalized_path)?;
        let mut uncovered_lines = Vec::new();
        let mut covered_count = 0usize;
        let mut changed_count = 0usize;

        if let Some(file_coverage) = file_coverage.as_deref() {
            for line in unique_lines {
                if !file_coverage.is_measured(line) {
                    continue;
                }
                if file_coverage.is_covered(line) {
                    covered_count += 1;
                } else {
                    uncovered_lines.push(line);
                }
            }

            changed_count = covered_count + uncovered_lines.len();
            total_changed += changed_count;
            total_covered += covered_count;
        } else if treat_missing_as_uncovered {
            uncovered_lines.extend(unique_lines.into_iter());
            changed_count = uncovered_lines.len();
            total_changed += changed_count;
        }

        if !uncovered_lines.is_empty() {
            uncovered_files.push(UncoveredFile {
                path: normalized_path,
                uncovered_lines,
                covered_lines: covered_count,
                changed_lines: changed_count,
            });
        }
    }

    Ok(CoverageReport {
        total_changed,
        total_covered,
        uncovered_files,
    })
}

#[cfg(test)]
mod tests {
    use super::analyze_changed_coverage;
    use crate::coverage::store::CoverageStore;
    use crate::coverage::CoverageSink;
    use crate::diff::types::ChangedFile;

    #[test]
    fn counts_unique_changed_lines_and_tracks_uncovered() {
        let changed_files = vec![ChangedFile {
            path: "src\\foo.rs".to_string(),
            changed_lines: vec![1, 1, 2, 3],
        }];

        let mut store = CoverageStore::default();
        store.on_line("src/foo.rs", 1, 2);
        store.on_line("src/foo.rs", 2, 0);

        let report = analyze_changed_coverage(&changed_files, &store, true).expect("report");

        assert_eq!(report.total_changed, 2);
        assert_eq!(report.total_covered, 1);
        assert_eq!(report.uncovered_files.len(), 1);
        let uncovered = &report.uncovered_files[0];
        assert_eq!(uncovered.path, "src/foo.rs");
        assert_eq!(uncovered.uncovered_lines, vec![2]);
    }

    #[test]
    fn treats_missing_files_as_uncovered() {
        let changed_files = vec![ChangedFile {
            path: "src/foo.rs".to_string(),
            changed_lines: vec![1, 2, 2],
        }];

        let store = CoverageStore::default();
        let report = analyze_changed_coverage(&changed_files, &store, true).expect("report");

        assert_eq!(report.total_changed, 2);
        assert_eq!(report.total_covered, 0);
        assert_eq!(report.uncovered_files.len(), 1);
        let uncovered = &report.uncovered_files[0];
        assert_eq!(uncovered.path, "src/foo.rs");
        assert_eq!(uncovered.uncovered_lines, vec![1, 2]);
    }

    #[test]
    fn ignores_missing_files_when_disabled() {
        let changed_files = vec![ChangedFile {
            path: "src/foo.rs".to_string(),
            changed_lines: vec![1, 2, 2],
        }];

        let store = CoverageStore::default();
        let report = analyze_changed_coverage(&changed_files, &store, false).expect("report");

        assert_eq!(report.total_changed, 0);
        assert_eq!(report.total_covered, 0);
        assert!(report.uncovered_files.is_empty());
    }
}
