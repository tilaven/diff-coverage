use std::path::PathBuf;

use super::error::AppError;
use super::output::OutputPlan;

pub(crate) fn validate_fail_under(
    fail_under: Option<f64>,
    diff_file: Option<&PathBuf>,
    coverage_files: &[PathBuf],
) -> Result<(), AppError> {
    let Some(threshold) = fail_under else {
        return Ok(());
    };

    if diff_file.is_none() || coverage_files.is_empty() {
        return Err(AppError::usage(
            "--fail-under requires both --diff-file and at least one coverage path",
        ));
    }
    if !(0.0..=100.0).contains(&threshold) {
        return Err(AppError::usage("--fail-under must be between 0 and 100"));
    }

    Ok(())
}

pub(crate) fn validate_output_requirements(
    output_plan: &OutputPlan,
    diff_file: Option<&PathBuf>,
    coverage_files: &[PathBuf],
) -> Result<(), AppError> {
    if output_plan.wants_non_cli() && (diff_file.is_none() || coverage_files.is_empty()) {
        return Err(AppError::usage(
            "--output requires both --diff-file and at least one coverage path",
        ));
    }

    Ok(())
}
