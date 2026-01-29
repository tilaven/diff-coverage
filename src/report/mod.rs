pub mod console;
pub mod gitlab_code_quality;
pub mod json;
pub mod summary;

use std::io::Write;

pub use console::CliReportGenerator;
pub use gitlab_code_quality::GitlabReportGenerator;
pub use json::JsonReportGenerator;
pub use summary::SummaryReportGenerator;

#[derive(Debug)]
pub struct CoverageReport {
    pub total_changed: usize,
    pub total_covered: usize,
    pub uncovered_files: Vec<UncoveredFile>,
}

#[derive(Debug)]
pub struct UncoveredFile {
    pub path: String,
    pub uncovered_lines: Vec<u32>,
    pub covered_lines: usize,
    pub changed_lines: usize,
}

impl CoverageReport {
    pub fn coverage_percent(&self) -> f64 {
        if self.total_changed == 0 {
            100.0
        } else {
            (self.total_covered as f64 / self.total_changed as f64) * 100.0
        }
    }
}

pub trait ReportGenerator {
    fn write_report(&self, report: &CoverageReport, out: &mut dyn Write) -> Result<(), String>;
}
