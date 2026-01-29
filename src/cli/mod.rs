use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::OnceLock;

use clap::{CommandFactory, FromArgMatches, Parser, ValueEnum, ValueHint};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ValueEnum)]
#[value(rename_all = "kebab_case")]
pub enum OutputFormat {
    Cli,
    Summary,
    Gitlab,
    Json,
}

pub const DEFAULT_OUTPUT_FORMAT: OutputFormat = OutputFormat::Cli;

impl OutputFormat {
    pub fn label(self) -> String {
        self.to_possible_value()
            .expect("output format possible value")
            .get_name()
            .to_string()
    }

    pub fn requires_path(self) -> bool {
        !matches!(self, OutputFormat::Cli | OutputFormat::Summary)
    }

    fn parse(raw: &str) -> Option<Self> {
        let raw = raw.trim();
        OutputFormat::value_variants()
            .iter()
            .copied()
            .find(|format| format.label().eq_ignore_ascii_case(raw))
    }

    fn labels() -> Vec<String> {
        OutputFormat::value_variants()
            .iter()
            .map(|format| format.label())
            .collect()
    }

    fn labels_without_path() -> Vec<String> {
        OutputFormat::value_variants()
            .iter()
            .copied()
            .filter(|format| !format.requires_path())
            .map(|format| format.label())
            .collect()
    }

    fn labels_requiring_path() -> Vec<String> {
        OutputFormat::value_variants()
            .iter()
            .copied()
            .filter(|format| format.requires_path())
            .map(|format| format.label())
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ValueEnum)]
#[value(rename_all = "kebab_case")]
pub enum MissingCoverageMode {
    Uncovered,
    Ignore,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputTarget {
    pub format: OutputFormat,
    pub path: Option<PathBuf>,
}

#[derive(Debug, Parser)]
#[command(
    name = "diff-cover",
    version,
    about = "Scan diffs for coverage changes.",
    arg_required_else_help = true
)]
pub struct CliOptions {
    #[arg(long, value_name = "PATH")]
    pub diff_file: Option<PathBuf>,
    #[arg(
        value_name = "COVERAGE",
        help = "Coverage file or directory; can be repeated or comma-separated",
        action = clap::ArgAction::Append,
        value_delimiter = ',',
        value_hint = ValueHint::AnyPath
    )]
    pub coverage_paths: Vec<PathBuf>,
    #[arg(long, value_name = "PERCENT")]
    pub fail_under: Option<f64>,
    #[arg(
        long = "missing-coverage",
        value_name = "MODE",
        default_value = "ignore",
        help = "How to handle files missing from coverage: uncovered or ignore"
    )]
    pub missing_coverage: MissingCoverageMode,
    #[arg(
        long = "output",
        id = "output",
        value_name = "FORMAT=PATH",
        help = "Output target(s); can be repeated or comma-separated",
        action = clap::ArgAction::Append,
        value_delimiter = ',',
        value_parser = parse_output_target
    )]
    pub outputs: Vec<OutputTarget>,
}

fn parse_output_target(raw: &str) -> Result<OutputTarget, String> {
    let (format_raw, path_raw) = match raw.split_once('=') {
        Some((format_raw, path_raw)) => (format_raw, Some(path_raw)),
        None => (raw, None),
    };
    let format = OutputFormat::parse(format_raw).ok_or_else(|| {
        format!(
            "output target format must be one of: {}",
            OutputFormat::labels().join(", ")
        )
    })?;
    match (format.requires_path(), path_raw) {
        (false, None) => Ok(OutputTarget { format, path: None }),
        (false, Some(_)) => Err(format!("{} output does not take a path", format.label())),
        (true, None) => Err(format!(
            "output target must be FORMAT=PATH for formats that require a path: {}",
            OutputFormat::labels_requiring_path().join(", ")
        )),
        (true, Some(path_raw)) if path_raw.is_empty() => {
            Err("output target path cannot be empty".to_string())
        }
        (true, Some(path_raw)) => Ok(OutputTarget {
            format,
            path: Some(PathBuf::from(path_raw)),
        }),
    }
}

fn output_help() -> &'static str {
    static OUTPUT_HELP: OnceLock<String> = OnceLock::new();
    OUTPUT_HELP.get_or_init(|| {
        let formats = OutputFormat::labels().join(", ");
        let no_path_formats = OutputFormat::labels_without_path().join(", ");
        let default_format = DEFAULT_OUTPUT_FORMAT.label();
        format!(
            "Output target(s); can be repeated or comma-separated [default: {default_format}] [possible values: {formats}] [formats that do not take a path: {no_path_formats}]"
        )
    })
    .as_str()
}

fn cli_command() -> clap::Command {
    let output_help = output_help();
    CliOptions::command().mut_arg("output", |arg| arg.help(output_help).long_help(output_help))
}

pub fn parse_args<I>(args: I) -> Result<CliOptions, String>
where
    I: IntoIterator<Item = OsString>,
{
    let matches = match cli_command().try_get_matches_from(args) {
        Ok(matches) => matches,
        Err(err) => {
            if matches!(
                err.kind(),
                clap::error::ErrorKind::DisplayHelp
                    | clap::error::ErrorKind::DisplayVersion
                    | clap::error::ErrorKind::MissingRequiredArgument
            ) {
                print!("{err}");
                std::process::exit(0);
            }
            return Err(err.to_string());
        }
    };

    CliOptions::from_arg_matches(&matches).map_err(|err| err.to_string())
}

pub fn print_help() {
    let mut cmd = cli_command();
    cmd.print_help().expect("print help");
    println!();
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use super::{parse_args, MissingCoverageMode, OutputFormat};

    #[test]
    fn parses_diff_file_flag() {
        let options = parse_args([
            OsString::from("bin"),
            OsString::from("--diff-file"),
            OsString::from("diff.txt"),
        ])
        .expect("parse");
        assert_eq!(options.diff_file.unwrap().to_string_lossy(), "diff.txt");
    }

    #[test]
    fn parses_diff_file_equals() {
        let options = parse_args([
            OsString::from("bin"),
            OsString::from("--diff-file=diff.txt"),
        ])
        .expect("parse");
        assert_eq!(options.diff_file.unwrap().to_string_lossy(), "diff.txt");
    }

    #[test]
    fn parses_coverage_paths_single() {
        let options =
            parse_args([OsString::from("bin"), OsString::from("cov.xml")]).expect("parse");
        assert_eq!(options.coverage_paths.len(), 1);
        assert_eq!(options.coverage_paths[0].to_string_lossy(), "cov.xml");
    }

    #[test]
    fn parses_coverage_paths_multiple() {
        let options = parse_args([
            OsString::from("bin"),
            OsString::from("a.xml"),
            OsString::from("b"),
        ])
        .expect("parse");
        assert_eq!(options.coverage_paths.len(), 2);
        assert_eq!(options.coverage_paths[0].to_string_lossy(), "a.xml");
        assert_eq!(options.coverage_paths[1].to_string_lossy(), "b");
    }

    #[test]
    fn parses_coverage_paths_comma_separated() {
        let options =
            parse_args([OsString::from("bin"), OsString::from("a.xml,b.xml")]).expect("parse");
        assert_eq!(options.coverage_paths.len(), 2);
        assert_eq!(options.coverage_paths[0].to_string_lossy(), "a.xml");
        assert_eq!(options.coverage_paths[1].to_string_lossy(), "b.xml");
    }

    #[test]
    fn parses_fail_under_flag() {
        let options = parse_args([
            OsString::from("bin"),
            OsString::from("--fail-under"),
            OsString::from("82.5"),
        ])
        .expect("parse");
        assert_eq!(options.fail_under, Some(82.5));
    }

    #[test]
    fn parses_output_target() {
        let options = parse_args([
            OsString::from("bin"),
            OsString::from("--output"),
            OsString::from("gitlab=code-quality.json"),
        ])
        .expect("parse");
        assert_eq!(options.outputs.len(), 1);
        assert_eq!(options.outputs[0].format, OutputFormat::Gitlab);
        assert_eq!(
            options.outputs[0].path.as_ref().unwrap().to_string_lossy(),
            "code-quality.json"
        );
    }

    #[test]
    fn parses_output_target_cli_without_path() {
        let options = parse_args([
            OsString::from("bin"),
            OsString::from("--output"),
            OsString::from("cli"),
        ])
        .expect("parse");
        assert_eq!(options.outputs.len(), 1);
        assert_eq!(options.outputs[0].format, OutputFormat::Cli);
        assert!(options.outputs[0].path.is_none());
    }

    #[test]
    fn parses_output_target_summary_without_path() {
        let options = parse_args([
            OsString::from("bin"),
            OsString::from("--output"),
            OsString::from("summary"),
        ])
        .expect("parse");
        assert_eq!(options.outputs.len(), 1);
        assert_eq!(options.outputs[0].format, OutputFormat::Summary);
        assert!(options.outputs[0].path.is_none());
    }

    #[test]
    fn parses_output_target_json() {
        let options = parse_args([
            OsString::from("bin"),
            OsString::from("--output"),
            OsString::from("json=report.json"),
        ])
        .expect("parse");
        assert_eq!(options.outputs.len(), 1);
        assert_eq!(options.outputs[0].format, OutputFormat::Json);
        assert_eq!(
            options.outputs[0].path.as_ref().unwrap().to_string_lossy(),
            "report.json"
        );
    }

    #[test]
    fn parses_missing_coverage_mode() {
        let options = parse_args([
            OsString::from("bin"),
            OsString::from("--diff-file"),
            OsString::from("diff.txt"),
            OsString::from("--missing-coverage"),
            OsString::from("ignore"),
        ])
        .expect("parse");
        assert_eq!(options.missing_coverage, MissingCoverageMode::Ignore);
    }
}
