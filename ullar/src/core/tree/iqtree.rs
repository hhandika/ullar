//! Species and gene tree inference using IQ-TREE.
use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
};

use crate::core::deps::iqtree::{IqTreeVersion, IQTREE2_EXE};

/// Default parameters for IQ-TREE 2
pub const DEFAULT_IQTREE2_PARAMS: &str = "-m GTR+G -B 1000";

/// Default parameters for IQ-TREE 1
pub const DEFAULT_IQTREE1_PARAMS: &str = "-m GTR+G -bb 1000";

pub struct IQTreeRunner<'a> {
    alignments: &'a [PathBuf],
    output_dir: &'a PathBuf,
    threads: Option<usize>,
    override_args: Option<&'a str>,
}

impl<'a> IQTreeRunner<'a> {
    pub fn new(
        alignments: &'a [PathBuf],
        output_dir: &'a PathBuf,
        threads: Option<usize>,
        override_args: Option<&'a str>,
    ) -> Self {
        Self {
            alignments,
            output_dir,
            threads,
            override_args,
        }
    }

    pub fn run(&self) {
        log::info!("Running IQ-TREE for {} alignment(s)", self.alignments.len());
        log::info!("Output directory: {}", self.output_dir.display());
        match self.threads {
            Some(threads) => log::info!("Threads: {}", threads),
            None => log::info!("Threads: auto"),
        }
        log::info!("Optional parameters: {:?}", self.override_args);
    }
}

pub struct IqTree<'a> {
    pub input_path: &'a Path,
    pub override_args: Option<&'a str>,
    pub version: &'a IqTreeVersion,
}

#[allow(dead_code)]
impl<'a> IqTree<'a> {
    fn new(path: &'a Path, override_args: Option<&'a str>, version: &'a IqTreeVersion) -> Self {
        Self {
            input_path: path,
            override_args,
            version,
        }
    }

    fn run_iqtree(&self, prefix: &str) -> Output {
        let mut out = Command::new(IQTREE2_EXE);
        out.arg("-s")
            .arg(self.input_path)
            .arg("--prefix")
            .arg(prefix);
        // self.get_thread_num(&mut out);
        // self.get_iqtree_params(&mut out);
        out.output().expect("Failed to run IQ-TREE")
    }

    // fn run_iqtree_concord(&self, prefix: &str) -> Output {
    //     let mut out = Command::new(IQTREE_EXE);
    //     out.arg("-t")
    //         .arg("concat.treefile")
    //         .arg("--gcf")
    //         .arg(GENE_TREE_NAME)
    //         .arg("-p")
    //         .arg(&self.path)
    //         .arg("--scf")
    //         .arg("100")
    //         .arg("--prefix")
    //         .arg(prefix)
    //         .output()
    //         .expect("Failed to run IQ-TREE concordance factors")
    // }

    // fn run_astral(&self) -> Output {
    //     let mut out = Command::new(ASTRAL_EXE);
    //     out.arg("-i")
    //         .arg(GENE_TREE_NAME)
    //         .arg("-o")
    //         .arg(ASTRAL_TREE_NAME)
    //         .output()
    //         .expect("Failed to run Astral")
    // }

    // fn get_iqtree_params(&self, out: &mut Command) {
    //     match self.params {
    //         Some(param) => {
    //             let params: Vec<&str> = param.split_whitespace().collect();
    //             params.iter().for_each(|param| {
    //                 out.arg(param);
    //             });
    //         }
    //         None => {
    //             out.arg("-B").arg("1000");
    //         }
    //     }
    // }

    // fn get_iqtree_files(&self, prefix: &str) -> Vec<PathBuf> {
    //     let pattern = format!("{}.*", prefix);
    //     self.get_files(&pattern)
    // }

    // fn get_thread_num(&self, out: &mut Command) {
    //     if self.params.is_none() {
    //         out.arg("-T").arg("1");
    //     }
    // }
}
