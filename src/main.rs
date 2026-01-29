mod app;
mod cli;
mod coverage;
mod diff;
mod report;
mod util;

use owo_colors::OwoColorize;

fn main() {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() == 1 {
        cli::print_help();
        return;
    }

    let options = match cli::parse_args(args) {
        Ok(options) => options,
        Err(message) => {
            eprintln!("{} {message}", "Error:".red().bold());
            std::process::exit(2);
        }
    };

    if let Err(err) = app::run(options) {
        eprintln!("{} {}", "Error:".red().bold(), err.message);
        std::process::exit(err.exit_code);
    }
}
