use std::io::IsTerminal;
use std::path::Path;

use crate::report::ReportGenerator;
use crate::{cli, report};

use super::error::AppError;
use super::output::OutputPlan;

pub(crate) fn write_reports(
    report: &report::CoverageReport,
    output_plan: &OutputPlan,
) -> Result<(), AppError> {
    for format in output_plan.order() {
        match format {
            cli::OutputFormat::Cli => {
                let mut stdout = std::io::stdout();
                let use_color = stdout.is_terminal();
                let generator = report::CliReportGenerator { use_color };
                if let Err(err) = generator.write_report(report, &mut stdout) {
                    return Err(AppError::usage(format!(
                        "Failed to write CLI report: {err}"
                    )));
                }
                eprintln!("Success: CLI report generated.");
            }
            cli::OutputFormat::Summary => {
                let mut stdout = std::io::stdout();
                let use_color = stdout.is_terminal();
                let generator = report::SummaryReportGenerator { use_color };
                if let Err(err) = generator.write_report(report, &mut stdout) {
                    return Err(AppError::usage(format!(
                        "Failed to write summary report: {err}"
                    )));
                }
                eprintln!("Success: summary report generated.");
            }
            cli::OutputFormat::Gitlab => {
                let path = output_plan
                    .path_for(format)
                    .expect("gitlab output path present");
                let generator = report::GitlabReportGenerator;
                write_report_to_path(&generator, report, path, "GitLab code quality")
                    .map_err(AppError::usage)?;
                eprintln!("Success: GitLab report written to {}.", path.display());
            }
            cli::OutputFormat::Json => {
                let path = output_plan
                    .path_for(format)
                    .expect("json output path present");
                let generator = report::JsonReportGenerator;
                write_report_to_path(&generator, report, path, "JSON").map_err(AppError::usage)?;
                eprintln!("Success: JSON report written to {}.", path.display());
            }
        }
    }

    Ok(())
}

fn write_report_to_path<G: ReportGenerator>(
    generator: &G,
    report: &report::CoverageReport,
    path: &Path,
    label: &str,
) -> Result<(), String> {
    let mut file = std::fs::File::create(path)
        .map_err(|err| format!("Failed to write {label} report {}: {err}", path.display()))?;
    generator
        .write_report(report, &mut file)
        .map_err(|err| format!("Failed to write {label} report {}: {err}", path.display()))
}
