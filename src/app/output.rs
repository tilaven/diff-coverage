use std::collections::HashMap;
use std::path::PathBuf;

use crate::cli;

use super::error::AppError;

pub(crate) struct OutputPlan {
    order: Vec<cli::OutputFormat>,
    paths: HashMap<cli::OutputFormat, PathBuf>,
}

impl OutputPlan {
    pub(crate) fn wants_non_cli(&self) -> bool {
        self.order
            .iter()
            .any(|format| !matches!(format, cli::OutputFormat::Cli | cli::OutputFormat::Summary))
    }

    pub(crate) fn order(&self) -> &[cli::OutputFormat] {
        &self.order
    }

    pub(crate) fn path_for(&self, format: &cli::OutputFormat) -> Option<&PathBuf> {
        self.paths.get(format)
    }
}

pub(crate) fn build_output_plan(
    output_targets: Vec<cli::OutputTarget>,
) -> Result<OutputPlan, AppError> {
    let mut output_paths = HashMap::new();
    let mut output_order = Vec::new();
    for target in output_targets {
        if output_order.contains(&target.format) {
            return Err(AppError::usage(format!(
                "--output {} can only be provided once",
                target.format.label()
            )));
        }
        match target.format {
            cli::OutputFormat::Cli | cli::OutputFormat::Summary => {
                if target.path.is_some() {
                    return Err(AppError::usage(format!(
                        "--output {} does not accept a path",
                        target.format.label()
                    )));
                }
            }
            _ => {
                let path = match target.path {
                    Some(path) => path,
                    None => {
                        return Err(AppError::usage(format!(
                            "--output {} requires a path",
                            target.format.label()
                        )));
                    }
                };
                output_paths.insert(target.format, path);
            }
        }
        output_order.push(target.format);
    }
    if output_order.is_empty() {
        output_order.push(cli::DEFAULT_OUTPUT_FORMAT);
    }

    Ok(OutputPlan {
        order: output_order,
        paths: output_paths,
    })
}
