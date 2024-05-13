mod cli;
mod core;
mod helper;
mod types;

fn main() {
    cli::cli::parse_commands();
}
