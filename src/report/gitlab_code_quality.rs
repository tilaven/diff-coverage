use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io::Write;

use serde::Serialize;

use super::{CoverageReport, ReportGenerator};

pub struct GitlabReportGenerator;

#[derive(Serialize)]
struct CodeQualityIssue {
    description: String,
    fingerprint: String,
    severity: String,
    location: IssueLocation,
}

#[derive(Serialize)]
struct IssueLocation {
    path: String,
    lines: IssueLines,
}

#[derive(Serialize)]
struct IssueLines {
    begin: u32,
    end: u32,
}

pub fn render_report(report: &CoverageReport) -> Result<String, String> {
    let mut issues = Vec::new();

    for file in &report.uncovered_files {
        for line in &file.uncovered_lines {
            issues.push(CodeQualityIssue {
                description: format!("Uncovered changed line {line}."),
                fingerprint: fingerprint_for(&file.path, *line),
                severity: "minor".to_string(),
                location: IssueLocation {
                    path: file.path.clone(),
                    lines: IssueLines {
                        begin: *line,
                        end: *line,
                    },
                },
            });
        }
    }

    let mut payload = serde_json::to_string_pretty(&issues).map_err(|err| err.to_string())?;
    payload.push('\n');
    Ok(payload)
}

impl ReportGenerator for GitlabReportGenerator {
    fn write_report(&self, report: &CoverageReport, out: &mut dyn Write) -> Result<(), String> {
        let payload = render_report(report)?;
        out.write_all(payload.as_bytes())
            .map_err(|err| err.to_string())
    }
}

fn fingerprint_for(path: &str, line: u32) -> String {
    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    line.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::GitlabReportGenerator;
    use crate::report::ReportGenerator;
    use crate::report::{CoverageReport, UncoveredFile};
    use serde_json::Value;

    #[test]
    fn writes_gitlab_code_quality_json() {
        let report = CoverageReport {
            total_changed: 2,
            total_covered: 0,
            uncovered_files: vec![UncoveredFile {
                path: "src/foo.rs".to_string(),
                uncovered_lines: vec![3, 7],
                covered_lines: 0,
                changed_lines: 2,
            }],
        };

        let mut out = Vec::new();
        GitlabReportGenerator
            .write_report(&report, &mut out)
            .expect("write report");

        let content = String::from_utf8(out).expect("utf8");
        assert!(content.ends_with('\n'));
        let payload: Vec<Value> = serde_json::from_str(&content).expect("parse json");
        assert_eq!(payload.len(), 2);

        let first = payload[0].as_object().expect("issue object");
        let description = first
            .get("description")
            .and_then(Value::as_str)
            .expect("description");
        assert!(description.contains("Uncovered changed line"));
        let severity = first
            .get("severity")
            .and_then(Value::as_str)
            .expect("severity");
        assert_eq!(severity, "minor");

        let fingerprint = first
            .get("fingerprint")
            .and_then(Value::as_str)
            .expect("fingerprint");
        assert_eq!(fingerprint.len(), 16);
        assert!(fingerprint.chars().all(|ch| ch.is_ascii_hexdigit()));

        let location = first
            .get("location")
            .and_then(Value::as_object)
            .expect("location");
        let path_value = location
            .get("path")
            .and_then(Value::as_str)
            .expect("location path");
        assert_eq!(path_value, "src/foo.rs");
        let lines = location
            .get("lines")
            .and_then(Value::as_object)
            .expect("lines");
        assert_eq!(lines.get("begin").and_then(Value::as_u64), Some(3));
        assert_eq!(lines.get("end").and_then(Value::as_u64), Some(3));
    }
}
