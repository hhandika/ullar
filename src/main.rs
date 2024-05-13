use cli::cli::Cli;

mod cli;
mod core;
mod helper;
mod types;

fn main() {
    Cli::new().parse();
}
