#![forbid(unsafe_code)]
use std::process::ExitCode;

mod localization;

fn main() -> ExitCode {
    let command = std::env::args().nth(1);
    match command.as_deref() {
        None | Some("--help" | "-h") => {
            println!("{}", localization::message("help"));
            ExitCode::SUCCESS
        }
        Some("probe" | "validate" | "extract" | "render-frame" | "export") => {
            eprintln!("{}", localization::message("not-implemented"));
            ExitCode::FAILURE
        }
        Some(other) => {
            eprintln!("{}: {other}", localization::message("unknown-command"));
            ExitCode::FAILURE
        }
    }
}
