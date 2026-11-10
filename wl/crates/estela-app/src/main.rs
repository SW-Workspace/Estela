#![forbid(unsafe_code)]
mod localization;

fn main() {
    eprintln!("{}", localization::message("app-pending"));
}
