//! Command-line interface for the binary ULLAR.
use clap::Parser;
use commands::{
    alignment::{AlignmentArgs, AlignmentInitArgs, AlignmentSubcommand},
    assembly::{AssemblyArgs, AssemblyInitArgs, AssemblySubcommand},
    clean::{ReadCleaningInitArgs, ReadCleaningSubcommand},
    deps::DepsSubcommand,
    map::MapSubcommand,
    tree::TreeInferenceSubcommand,
    utils::{ScannerSubcommand, UtilSubCommand},
    UllarCli, UllarSubcommand,
};
use segul::helper::utils;
use std::{path::PathBuf, time::Instant};

use crate::{
    core::{
        alignment::{init::AlignmentInit, SequenceAlignment},
        assembly::{init::AssemblyInit, Assembly},
        clean::{init::ReadCleaningInit, ReadCleaner},
        map::{init::InitMappingConfig, ContigMapping},
        tree::{init::TreeInferenceInit, TreeEstimation},
        utils::{checksum::Sha256Executor, scan::ReadScanner},
    },
    deps::DependencyCheck,
    helper::{self, common::PrettyHeader},
};

#[cfg(target_family = "unix")]
use crate::core::utils::symlinks::Symlinks;

pub mod commands;

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
        let logger = self.init_logger();
        PrettyHeader::new().get_welcome_header();
        self.match_subcommand();
        let elapsed = time.elapsed();
        println!();
        log::info!("{:18}: {}", "Log file", logger.display());
        utils::print_execution_time(elapsed);
    }

    fn init_logger(&self) -> PathBuf {
        helper::logs::init_logger(
            self.command.log_dir.as_ref(),
            self.command.log_prefix.as_str(),
        )
    }

    fn match_subcommand(&self) {
        match &self.command.sub_cmd {
            UllarSubcommand::New(_) => unimplemented!("New project is not yet implemented"),
            UllarSubcommand::Clean(subcommand) => CleanArgParser::new(subcommand).parse(),
            UllarSubcommand::Assemble(subcommand) => AssemblyArgParser::new(subcommand).parse(),
            UllarSubcommand::Map(subcommand) => MapArgParser::new(subcommand).parse(),
            UllarSubcommand::Alignment(subcommand) => AlignmentArgParser::new(subcommand).parse(),
            UllarSubcommand::Tree(subcommand) => TreeArgParser::new(subcommand).parse(),
            UllarSubcommand::Deps(subcommand) => self.parse_dependencies(subcommand),
            UllarSubcommand::Utils(util_args) => self.parse_utils(util_args),
        }
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
            UtilSubCommand::Symlink(args) => Symlinks::from_arg(args).create(),
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

struct CleanArgParser<'a> {
    subcommand: &'a ReadCleaningSubcommand,
}

impl<'a> CleanArgParser<'a> {
    fn new(subcommand: &'a ReadCleaningSubcommand) -> Self {
        Self { subcommand }
    }

    fn parse(&self) {
        match self.subcommand {
            ReadCleaningSubcommand::Init(init_args) => self.init(init_args),
            ReadCleaningSubcommand::Clean(clean_args) => {
                let cleaner = ReadCleaner::from_arg(clean_args);
                cleaner.clean();
            }
        }
    }

    fn init(&self, args: &ReadCleaningInitArgs) {
        let mut init = ReadCleaningInit::from_arg(args);
        init.init().expect("Failed to execute new command");
    }
}

struct AssemblyArgParser<'a> {
    subcommand: &'a AssemblySubcommand,
}

impl<'a> AssemblyArgParser<'a> {
    fn new(subcommand: &'a AssemblySubcommand) -> Self {
        Self { subcommand }
    }

    fn parse(&self) {
        match self.subcommand {
            AssemblySubcommand::Init(init_args) => self.init(init_args),
            AssemblySubcommand::Assembly(assembly_args) => self.assemble(assembly_args),
        }
    }

    fn init(&self, args: &AssemblyInitArgs) {
        AssemblyInit::from_arg(args)
            .init()
            .expect("Failed to execute new command");
    }

    fn assemble(&self, args: &AssemblyArgs) {
        Assembly::from_arg(args).assemble();
    }
}

struct MapArgParser<'a> {
    subcommand: &'a MapSubcommand,
}

impl<'a> MapArgParser<'a> {
    fn new(subcommand: &'a MapSubcommand) -> Self {
        Self { subcommand }
    }

    fn parse(&self) {
        match self.subcommand {
            MapSubcommand::Init(args) => InitMappingConfig::from_arg(args).init(),
            MapSubcommand::Contig(args) => ContigMapping::from_arg(args).map(),
            MapSubcommand::Read(_) => {
                unimplemented!("Map read command is not yet implemented")
            }
        }
    }
}

struct AlignmentArgParser<'a> {
    subcommand: &'a AlignmentSubcommand,
}

impl<'a> AlignmentArgParser<'a> {
    fn new(subcommand: &'a AlignmentSubcommand) -> Self {
        Self { subcommand }
    }

    fn parse(&self) {
        match self.subcommand {
            AlignmentSubcommand::Init(init_args) => self.init(init_args),
            AlignmentSubcommand::Align(run_args) => self.run(run_args),
        }
    }

    fn init(&self, args: &AlignmentInitArgs) {
        AlignmentInit::new(args).init();
    }

    fn run(&self, args: &AlignmentArgs) {
        SequenceAlignment::from_arg(args).align();
    }
}

struct TreeArgParser<'a> {
    subcommand: &'a TreeInferenceSubcommand,
}
impl<'a> TreeArgParser<'a> {
    fn new(subcommand: &'a TreeInferenceSubcommand) -> Self {
        Self { subcommand }
    }

    fn parse(&self) {
        match self.subcommand {
            TreeInferenceSubcommand::Init(args) => TreeInferenceInit::from_arg(args).init(),
            TreeInferenceSubcommand::Run(args) => {
                TreeEstimation::from_arg(args).infer();
            }
        }
    }
}
