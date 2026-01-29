mod error;
mod files;
mod output;
mod reporting;
mod validation;

use crate::{cli, coverage};
use error::AppError;
use files::{collect_coverage_files, load_changed_files, load_coverage_files};
use output::build_output_plan;
use reporting::write_reports;
use validation::{validate_fail_under, validate_output_requirements};

pub fn run(options: cli::CliOptions) -> Result<(), AppError> {
    let diff_file = options.diff_file;
    let coverage_paths = options.coverage_paths;
    let fail_under = options.fail_under;
    let output_targets = options.outputs;
    let missing_coverage = options.missing_coverage;

    let coverage_files = collect_coverage_files(coverage_paths).map_err(AppError::usage)?;

    validate_fail_under(fail_under, diff_file.as_ref(), &coverage_files)?;

    let output_plan = build_output_plan(output_targets)?;
    validate_output_requirements(&output_plan, diff_file.as_ref(), &coverage_files)?;

    match (diff_file, coverage_files.is_empty()) {
        (Some(diff_file), false) => {
            let changed = load_changed_files(&diff_file).map_err(|err| {
                AppError::usage(format!(
                    "Failed to parse diff file {}: {err}",
                    diff_file.display()
                ))
            })?;

            let coverage = load_coverage_files(&coverage_files)
                .map_err(|err| AppError::usage(format!("Failed to parse coverage files: {err}")))?;

            let treat_missing_as_uncovered =
                matches!(missing_coverage, cli::MissingCoverageMode::Uncovered);
            let report =
                coverage::analyze_changed_coverage(&changed, &coverage, treat_missing_as_uncovered)
                    .map_err(|err| AppError::usage(err.to_string()))?;
            write_reports(&report, &output_plan)?;

            if let Some(threshold) = fail_under {
                let percent = report.coverage_percent();
                if percent < threshold {
                    return Err(AppError::fail_under(percent, threshold));
                }
            }
        }
        (Some(diff_file), true) => {
            let changed = load_changed_files(&diff_file).map_err(|err| {
                AppError::usage(format!(
                    "Failed to parse diff file {}: {err}",
                    diff_file.display()
                ))
            })?;

            for file in changed {
                println!("{}: {:?}", file.path, file.changed_lines);
            }
        }
        (None, false) => {
            return Err(AppError::usage("coverage path(s) require --diff-file"));
        }
        (None, true) => {
            cli::print_help();
        }
    }

    Ok(())
}
