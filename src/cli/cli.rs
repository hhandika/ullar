use clap::Parser;

use super::args::{SubCommand, UllarCli};

pub fn parse_commands() {
    let args = UllarCli::parse();
    match args.sub_cmd {
        SubCommand::Init(new_args) => {
            println!("Init subcommand selected with directory: {}", new_args.dir);
        }
    }
}
