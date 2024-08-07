//! Command-line interface for ULLAR project.
pub mod commands;

use clap::Parser;
use commands::{
    deps::DepsSubcommand,
    utils::{ScannerSubcommand, UtilSubCommand},
    UllarCli, UllarSubcommand,
};
use segul::helper::utils;
use std::time::Instant;

use crate::{
    core::{
        assembly::Assembly,
        init::new::NewProject,
        qc::ReadCleaner,
        tree::TreeEstimation,
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
            UllarSubcommand::New(new_args) => {
                let mut parser = NewProject::from_arg(new_args);
                parser.execute().expect("Failed to execute new command");
            }
            UllarSubcommand::Init(_) => unimplemented!("Init command is not yet implemented"),
            UllarSubcommand::Clean(clean_args) => {
                let cleaner = ReadCleaner::from_arg(clean_args);
                cleaner.clean();
            }
            UllarSubcommand::Assemble(assembly_args) => {
                let assembly = Assembly::from_arg(assembly_args);
                assembly.assemble();
            }
            UllarSubcommand::Map => unimplemented!("Map command is not yet implemented"),
            UllarSubcommand::Tree(tree_args) => TreeEstimation::from_arg(tree_args).run(),
            UllarSubcommand::Deps(subcommand) => self.parse_dependencies(subcommand),

            UllarSubcommand::Utils(util_args) => self.parse_utils(util_args),
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
            _ => unimplemented!("Dependency UllarSubcommand is not yet implemented"),
        }
    }

    fn parse_utils(&self, util_args: &UtilSubCommand) {
        match util_args {
            UtilSubCommand::Checksum(sha256_args) => {
                let parser = Sha256Executor::from_arg(sha256_args);
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
                let parser = ReadScanner::from_arg(read_args);
                parser.scan().expect("Failed to execute read scan command");
            }
        }
    }
}
