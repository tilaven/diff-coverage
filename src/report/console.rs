use std::io::{BufWriter, Write};

use owo_colors::OwoColorize;

use super::{CoverageReport, ReportGenerator};

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

    let mut out = BufWriter::new(out);
    writeln!(
        out,
        "Changed lines covered: {}/{}",
        report.total_covered, report.total_changed
    )?;

    if report.uncovered_files.is_empty() {
        let message = "All changed lines are covered.";
        if use_color {
            writeln!(out, "{}", message.green().bold())?;
        } else {
            writeln!(out, "{message}")?;
        }
    } else {
        writeln!(out)?;
        writeln!(out, "Uncovered files:")?;
        for file in &report.uncovered_files {
            let percent = if file.changed_lines == 0 {
                100.0
            } else {
                (file.covered_lines as f64 / file.changed_lines as f64) * 100.0
            };
            let percent_text = format!("{percent:.2}%");
            let percent_display = if use_color {
                colorize_percent(&percent_text, percent)
            } else {
                percent_text
            };
            let lines = format_line_ranges(&file.uncovered_lines);
            writeln!(out, "{} ({}): {}", file.path, percent_display, lines)?;
        }
    }

    if !report.skipped_files.is_empty() {
        writeln!(out)?;
        let header =
            "Skipped paths from uncovered line calculation (matched --skip-coverage-path):";
        if use_color {
            writeln!(out, "{}", header.yellow().bold())?;
        } else {
            writeln!(out, "{header}")?;
        }
        for skipped in &report.skipped_files {
            writeln!(out, "- {} (matched {})", skipped.path, skipped.pattern)?;
        }
    }

    writeln!(out)?;
    writeln!(out, "-------------")?;
    writeln!(out, "Coverage for changed lines: {percent_display}")?;
    out.flush()
}

pub struct CliReportGenerator {
    pub use_color: bool,
}

impl ReportGenerator for CliReportGenerator {
    fn write_report(&self, report: &CoverageReport, out: &mut dyn Write) -> Result<(), String> {
        render_to(report, out, self.use_color).map_err(|err| err.to_string())
    }
}

pub(crate) fn colorize_percent(text: &str, percent: f64) -> String {
    if percent >= 90.0 {
        text.green().bold().to_string()
    } else if percent >= 70.0 {
        text.yellow().bold().to_string()
    } else {
        text.red().bold().to_string()
    }
}

fn format_line_ranges(lines: &[u32]) -> String {
    if lines.is_empty() {
        return String::new();
    }

    let mut ranges = Vec::new();
    let mut start = lines[0];
    let mut end = lines[0];

    for &line in lines.iter().skip(1) {
        if line == end + 1 {
            end = line;
        } else {
            push_range(&mut ranges, start, end);
            start = line;
            end = line;
        }
    }

    push_range(&mut ranges, start, end);
    ranges.join(", ")
}

fn push_range(ranges: &mut Vec<String>, start: u32, end: u32) {
    if start == end {
        ranges.push(start.to_string());
    } else {
        ranges.push(format!("{start}-{end}"));
    }
}

#[cfg(test)]
mod tests {
    use super::{format_line_ranges, render_to};
    use crate::report::{CoverageReport, SkippedFile};

    #[test]
    fn formats_line_ranges() {
        assert_eq!(format_line_ranges(&[1, 3, 5]), "1, 3, 5");
        assert_eq!(format_line_ranges(&[1, 2, 5]), "1-2, 5");
        assert_eq!(format_line_ranges(&[1, 2, 3, 5]), "1-3, 5");
        assert_eq!(format_line_ranges(&[1, 3, 5, 6, 7, 8, 9]), "1, 3, 5-9");
    }

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
