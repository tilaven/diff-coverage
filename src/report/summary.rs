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
