//! Command-line interface for ULLAR project.
pub mod args;

use args::DepsSubcommand;
use clap::Parser;
use segul::helper::utils;

use std::time::Instant;

use self::args::{ScannerSubcommand, SubCommand, UllarCli, UtilSubCommand};
use crate::{
    core::{
        assembly::Assembly,
        init::new::NewExecutor,
        qc::ReadCleaner,
        utils::{checksum::Sha256Executor, deps::DependencyCheck, scan::ReadScanner},
    },
    helper::{self, common::PrettyHeader},
};

#[cfg(target_family = "unix")]
use crate::core::utils::symlinks::Symlinks;

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
        PrettyHeader::new().get_welcome_header();
        match &self.command.sub_cmd {
            SubCommand::New(new_args) => {
                let mut parser = NewExecutor::new(new_args);
                parser.execute().expect("Failed to execute new command");
            }
            SubCommand::Init(_) => unimplemented!("Init command is not yet implemented"),
            SubCommand::Clean(clean_args) => {
                let cleaner = ReadCleaner::new(clean_args);
                cleaner.clean();
            }
            SubCommand::Assemble(assembly_args) => {
                let assembly = Assembly::new(assembly_args);
                assembly.assemble();
            }
            SubCommand::Map => unimplemented!("Map command is not yet implemented"),
            SubCommand::Deps(subcommand) => self.parse_dependencies(subcommand),

            SubCommand::Utils(util_args) => self.parse_utils(util_args),
        }
        let elapsed = time.elapsed();
        println!();
        log::info!("{:18}: {}", "Log file", logger.display());
        utils::print_execution_time(elapsed);
    }

    fn parse_dependencies(&self, deps_subcommand: &DepsSubcommand) {
        match deps_subcommand {
            DepsSubcommand::Check => {
                let mut deps = DependencyCheck::new();
                deps.check();
            }
            _ => unimplemented!("Dependency subcommand is not yet implemented"),
        }
    }

    fn parse_utils(&self, util_args: &UtilSubCommand) {
        match util_args {
            UtilSubCommand::Checksum(sha256_args) => {
                let parser = Sha256Executor::new(sha256_args);
                parser.execute().expect("Failed to execute sha256 command");
            }
            UtilSubCommand::Scan(scan_subcommand) => self.parse_read_scan(scan_subcommand),
            #[cfg(target_family = "unix")]
            UtilSubCommand::Symlink(args) => Symlinks::new(args).create(),
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
