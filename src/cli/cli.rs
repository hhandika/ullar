use clap::Parser;
use segul::helper::utils;

use std::time::Instant;

use super::args::{SubCommand, UllarCli};
use crate::{core::new::NewExecutor, helper};

pub fn parse_commands() {
    let time = Instant::now();

    let args = UllarCli::parse();
    let logger = helper::logs::init_logger(args.log_dir.as_ref(), args.log_prefix.as_str());
    match args.sub_cmd {
        SubCommand::New(new_args) => {
            let parser = NewExecutor::new(&new_args);
            parser.execute().expect("Failed to execute new command");
        }
        SubCommand::Util(_) => {
            log::warn!("Utility functions are not implemented yet");
        }
    }
    let elapsed = time.elapsed();
    println!();
    log::info!("{:18}: {}", "Log file", logger.display());
    utils::print_execution_time(elapsed);
}
