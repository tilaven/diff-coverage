use std::io::Write;

use serde::Serialize;

use super::{CoverageReport, ReportGenerator};

#[derive(Serialize)]
struct JsonReport {
    total_changed: usize,
    total_covered: usize,
    coverage_percent: f64,
    uncovered_files: Vec<JsonFile>,
}

#[derive(Serialize)]
struct JsonFile {
    path: String,
    uncovered_lines: Vec<u32>,
}

pub struct JsonReportGenerator;

impl ReportGenerator for JsonReportGenerator {
    fn write_report(&self, report: &CoverageReport, out: &mut dyn Write) -> Result<(), String> {
        let payload = render_report(report)?;
        out.write_all(payload.as_bytes())
            .map_err(|err| err.to_string())
    }
}

fn render_report(report: &CoverageReport) -> Result<String, String> {
    let uncovered_files = report
        .uncovered_files
        .iter()
        .map(|file| JsonFile {
            path: file.path.clone(),
            uncovered_lines: file.uncovered_lines.clone(),
        })
        .collect();
    let payload = JsonReport {
        total_changed: report.total_changed,
        total_covered: report.total_covered,
        coverage_percent: report.coverage_percent(),
        uncovered_files,
    };
    let mut text = serde_json::to_string_pretty(&payload).map_err(|err| err.to_string())?;
    text.push('\n');
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::JsonReportGenerator;
    use crate::report::ReportGenerator;
    use crate::report::{CoverageReport, UncoveredFile};
    use serde_json::Value;

    #[test]
    fn writes_simple_json_report() {
        let report = CoverageReport {
            total_changed: 4,
            total_covered: 2,
            uncovered_files: vec![UncoveredFile {
                path: "src/foo.rs".to_string(),
                uncovered_lines: vec![2, 3],
                covered_lines: 2,
                changed_lines: 4,
            }],
        };

        let mut out = Vec::new();
        JsonReportGenerator
            .write_report(&report, &mut out)
            .expect("write report");

        let content = String::from_utf8(out).expect("utf8");
        assert!(content.ends_with('\n'));
        let payload: Value = serde_json::from_str(&content).expect("parse json");
        assert_eq!(payload["total_changed"], 4);
        assert_eq!(payload["total_covered"], 2);
        assert!(payload["coverage_percent"].as_f64().is_some());
        assert_eq!(payload["uncovered_files"][0]["path"], "src/foo.rs");
        assert_eq!(payload["uncovered_files"][0]["uncovered_lines"][0], 2);
    }
}
