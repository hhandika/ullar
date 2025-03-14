//! Multi-species coalescent model tree estimation using ASTER

use std::{
    fs::create_dir_all,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use crate::{
    core::deps::DepMetadata, helper::common, parse_override_args, types::trees::MscInferenceMethod,
};

use super::configs::AsterParams;

const ASTRAL_FNAME: &str = "astral";
const ASTRAL_PRO_FNAME: &str = "astral_pro";
const WASTRAL_FNAME: &str = "wastral";
const TREE_EXTENSION: &str = "tre";

const LOG_EXTENSION: &str = "log";

pub struct MscAster<'a> {
    configs: &'a AsterParams,
    gene_trees: &'a Path,
    output_dir: &'a Path,
}

impl<'a> MscAster<'a> {
    pub fn new(configs: &'a AsterParams, gene_trees: &'a Path, output_dir: &'a Path) -> Self {
        Self {
            configs,
            gene_trees,
            output_dir,
        }
    }

    pub fn infer(&self) {
        create_dir_all(self.output_dir).expect("Failed to create output directory.");
        let progress_bar = common::init_progress_bar(self.configs.methods.len() as u64);
        progress_bar.set_message("msc trees");
        self.configs.methods.iter().for_each(|(method, dep)| {
            self.run_aster(method, dep.as_ref());
            progress_bar.inc(1);
        });
        progress_bar.finish_with_message("msc trees\n");
    }

    fn run_aster(&self, method: &MscInferenceMethod, dep: Option<&DepMetadata>) {
        match dep {
            Some(dep) => {
                let output_path = self.get_output_path(method);
                let runner = AsterRunner::new(dep, self.gene_trees, None);
                let output = runner.run(&output_path).expect("Failed to run ASTRAL.");
                self.log_output(method, output);
            }
            None => panic!("Astral dependency not found. Please check the configuration."),
        }
    }

    fn log_output(&self, method: &MscInferenceMethod, output: Output) {
        let log_path = self.get_log_path(method);
        std::fs::write(log_path, output.stderr).expect("Failed to write log file.");
    }

    fn get_log_path(&self, method: &MscInferenceMethod) -> PathBuf {
        match method {
            MscInferenceMethod::Astral => self
                .output_dir
                .join(ASTRAL_FNAME)
                .with_extension(LOG_EXTENSION),
            MscInferenceMethod::AstralPro => self
                .output_dir
                .join(ASTRAL_PRO_FNAME)
                .with_extension(LOG_EXTENSION),
            MscInferenceMethod::WeightedAstral => self
                .output_dir
                .join(WASTRAL_FNAME)
                .with_extension(LOG_EXTENSION),
        }
    }

    fn get_output_path(&self, method: &MscInferenceMethod) -> PathBuf {
        match method {
            MscInferenceMethod::Astral => self
                .output_dir
                .join(ASTRAL_FNAME)
                .with_extension(TREE_EXTENSION),
            MscInferenceMethod::AstralPro => self
                .output_dir
                .join(ASTRAL_PRO_FNAME)
                .with_extension(TREE_EXTENSION),
            MscInferenceMethod::WeightedAstral => self
                .output_dir
                .join(WASTRAL_FNAME)
                .with_extension(TREE_EXTENSION),
        }
    }
}

pub struct AsterRunner<'a> {
    pub dependency: &'a DepMetadata,
    pub gene_trees: &'a Path,
    pub optional_args: Option<&'a str>,
}

impl<'a> AsterRunner<'a> {
    pub fn new(
        dependency: &'a DepMetadata,
        gene_trees: &'a Path,
        optional_args: Option<&'a str>,
    ) -> Self {
        Self {
            dependency,
            gene_trees,
            optional_args,
        }
    }

    pub fn run(&self, output_path: &Path) -> Result<Output, std::io::Error> {
        let executable = self.get_executable();
        let mut out = Command::new(executable);

        out.arg("-i")
            .arg(self.gene_trees)
            .arg("-o")
            .arg(output_path);

        if let Some(opt_args) = &self.optional_args {
            parse_override_args!(out, opt_args);
        }

        out.output()
    }

    fn get_executable(&self) -> String {
        match &self.dependency.executable {
            Some(executable) => executable.to_string(),
            None => panic!(
                "ASTRAL Executable not found. \
            Please check ASTRAL is installed \
            and the path is set correctly."
            ),
        }
    }
}
