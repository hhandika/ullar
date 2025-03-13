//! Multi-species coalescent model tree estimation using ASTER

use std::{path::Path, process::Command};

use crate::{core::deps::DepMetadata, parse_override_args, types::trees::MscInferenceMethod};

use super::configs::AsterParams;
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
        self.configs
            .methods
            .iter()
            .for_each(|(method, dep)| match method {
                MscInferenceMethod::Astral => self.run_astral(dep.as_ref()),
                _ => unimplemented!(),
            });
    }

    fn run_astral(&self, dep: Option<&DepMetadata>) {
        match dep {
            Some(dep) => {
                let runner = AstralRunner::new(dep, self.gene_trees, None);
                runner.run(self.output_dir);
            }
            None => panic!("Astral dependency not found. Please check the configuration."),
        }
    }
}

pub struct AstralRunner<'a> {
    pub dependency: &'a DepMetadata,
    pub gene_trees: &'a Path,
    pub optional_args: Option<&'a str>,
}

impl<'a> AstralRunner<'a> {
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

    pub fn run(&self, output_dir: &Path) {
        let executable = self.get_executable();
        let mut out = Command::new(executable);

        out.arg("-i").arg(self.gene_trees).arg("-o").arg(output_dir);

        if let Some(opt_args) = &self.optional_args {
            parse_override_args!(out, opt_args);
        }
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
