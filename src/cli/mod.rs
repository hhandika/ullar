//! Command-line interface for ULLAR project.
pub mod args;

use clap::Parser;
use segul::helper::utils;

use std::time::Instant;

use self::args::{ScannerSubcommand, SubCommand, UllarCli, UtilSubCommand};
use crate::{
    core::{
        new::NewExecutor,
        utils::{scan::ReadScanner, sha256::Sha256Executor},
    },
    helper,
};
pub struct Cli {
    pub command: UllarCli,
}

impl Default for Cli {
    fn default() -> Self {
        Self::new()
    }
}

impl Cli {
    pub fn new() -> Self {
        Self {
            command: UllarCli::parse(),
        }
    }

    pub fn parse(&mut self) {
        let time = Instant::now();
        let logger = helper::logs::init_logger(
            self.command.log_dir.as_ref(),
            self.command.log_prefix.as_str(),
        );
        match &self.command.sub_cmd {
            SubCommand::New(new_args) => {
                let mut parser = NewExecutor::new(new_args);
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
                let parser = Sha256Executor::new(sha256_args);
                parser.execute().expect("Failed to execute sha256 command");
            }
            UtilSubCommand::ScanSubCommand(scan_subcommand) => {
                self.parse_read_scan(scan_subcommand)
            }
        }
    }

    fn parse_read_scan(&self, scan_args: &ScannerSubcommand) {
        match scan_args {
            ScannerSubcommand::ReadSubCommand(read_args) => {
                let parser = ReadScanner::new(read_args);
                parser.scan().expect("Failed to execute read scan command");
            }
        }
    }
}
