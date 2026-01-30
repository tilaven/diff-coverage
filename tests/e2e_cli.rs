use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn fixture_path(relative: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn unique_report_path() -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "diff_coverage_e2e_{}_{}.json",
        std::process::id(),
        nanos
    ))
}

#[test]
fn e2e_cli_reports_changed_line_totals() {
    let diff_path = fixture_path("tests/fixtures/git_diff.diff");
    let coverage_path = fixture_path("tests/fixtures/coverage_clover.xml");
    let report_path = unique_report_path();

    let output_target = format!("json={}", report_path.display());
    let output = Command::new(env!("CARGO_BIN_EXE_diff-coverage"))
        .arg("--diff-file")
        .arg(&diff_path)
        .arg(&coverage_path)
        .arg("--output")
        .arg(output_target)
        .output()
        .expect("run diff-coverage");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let report_text = std::fs::read_to_string(&report_path).expect("read report");
    let payload: serde_json::Value = serde_json::from_str(&report_text).expect("parse json");

    assert_eq!(payload["total_changed"], 2);
    assert_eq!(payload["total_covered"], 1);
    let percent = payload["coverage_percent"].as_f64().expect("percent");
    assert!(
        (percent - 50.0).abs() < 0.001,
        "coverage_percent = {percent}"
    );

    let _ = std::fs::remove_file(&report_path);
}

#[test]
fn e2e_cli_reports_changed_line_totals_for_same_name() {
    let diff_path = fixture_path("tests/fixtures/same_name/same_name_diff.text");
    let coverage_path = fixture_path("tests/fixtures/same_name/same_name_coverage_cobertura.xml");
    let report_path = unique_report_path();

    let output_target = format!("json={}", report_path.display());
    let output = Command::new(env!("CARGO_BIN_EXE_diff-coverage"))
        .arg("--diff-file")
        .arg(&diff_path)
        .arg(&coverage_path)
        .arg("--output")
        .arg(output_target)
        .output()
        .expect("run diff-coverage");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let report_text = std::fs::read_to_string(&report_path).expect("read report");
    let payload: serde_json::Value = serde_json::from_str(&report_text).expect("parse json");

    let percent = payload["coverage_percent"].as_f64().expect("percent");
    assert!(
        (percent - 50.0).abs() < 0.001,
        "coverage_percent = {percent}"
    );
    assert_eq!(payload["total_changed"], 2);
    assert_eq!(payload["total_covered"], 1);

    let _ = std::fs::remove_file(&report_path);
}
