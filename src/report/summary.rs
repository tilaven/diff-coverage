use std::io::Write;

use super::{CoverageReport, ReportGenerator};
use crate::report::console::colorize_percent;

pub fn render_to<W: Write + ?Sized>(
    report: &CoverageReport,
    out: &mut W,
    use_color: bool,
) -> std::io::Result<()> {
    let percent = report.coverage_percent();
    let percent_text = format!("{percent:.2}%");
    let percent_display = if use_color {
        colorize_percent(&percent_text, percent)
    } else {
        percent_text
    };

    writeln!(
        out,
        "Summary: {percent_display} ({}/{}) changed lines covered",
        report.total_covered, report.total_changed
    )?;
    if !report.skipped_files.is_empty() {
        writeln!(
            out,
            "Skipped paths from uncovered line calculation (matched --skip-coverage-path):"
        )?;
        for skipped in &report.skipped_files {
            writeln!(out, "- {} (matched {})", skipped.path, skipped.pattern)?;
        }
    }

    Ok(())
}

pub struct SummaryReportGenerator {
    pub use_color: bool,
}

impl ReportGenerator for SummaryReportGenerator {
    fn write_report(&self, report: &CoverageReport, out: &mut dyn Write) -> Result<(), String> {
        render_to(report, out, self.use_color).map_err(|err| err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::render_to;
    use crate::report::{CoverageReport, SkippedFile};

    #[test]
    fn includes_skipped_paths() {
        let report = CoverageReport {
            total_changed: 1,
            total_covered: 1,
            uncovered_files: Vec::new(),
            skipped_files: vec![SkippedFile {
                path: "pkg/mongodb/client.go".to_string(),
                pattern: "^pkg/mongodb".to_string(),
            }],
        };

        let mut out = Vec::new();
        render_to(&report, &mut out, false).expect("render");
        let text = String::from_utf8(out).expect("utf8");

        assert!(text.contains("Skipped paths"));
        assert!(text.contains("pkg/mongodb/client.go"));
        assert!(text.contains("^pkg/mongodb"));
    }
}
