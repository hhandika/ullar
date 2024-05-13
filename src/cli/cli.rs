use clap::Parser;
use segul::helper::utils;

use std::time::Instant;

use super::args::{SubCommand, UllarCli, UtilSubCommand};
use crate::{
    core::{new::NewExecutor, utils::sha256::Sha256Executor},
    helper,
};

pub struct Cli {
    pub command: UllarCli,
}

impl Cli {
    pub fn new() -> Self {
        Self {
            command: UllarCli::parse(),
        }
    }

    pub fn parse(&self) {
        let time = Instant::now();
        let logger = helper::logs::init_logger(
            self.command.log_dir.as_ref(),
            self.command.log_prefix.as_str(),
        );
        match &self.command.sub_cmd {
            SubCommand::New(new_args) => {
                let parser = NewExecutor::new(&new_args);
                parser.execute().expect("Failed to execute new command");
            }
            SubCommand::Utils(util_args) => self.parse_utils(util_args),
        }
        let elapsed = time.elapsed();
        println!();
        log::info!("{:18}: {}", "Log file", logger.display());
        utils::print_execution_time(elapsed);
    }

    fn parse_utils(&self, util_args: &UtilSubCommand) {
        match util_args {
            UtilSubCommand::Sha256SubCommand(sha256_args) => {
                let parser = Sha256Executor::new(&sha256_args);
                parser.execute().expect("Failed to execute sha256 command");
            }
        }
    }
}
