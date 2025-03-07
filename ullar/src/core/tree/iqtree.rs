//! Species and gene tree inference using IQ-TREE.
use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
};

use segul::{
    helper::{
        concat::Concat,
        files,
        types::{DataType, InputFmt, OutputFmt, PartitionFmt},
    },
    writer::{partition::PartWriter, sequences::SeqWriter},
};

use crate::{
    core::deps::{iqtree::IQTREE2_EXE, DepMetadata},
    helper::{common, files::PathCheck},
    types::alignments::AlignmentFiles,
};

use super::configs::IqTreeConfig;

const SPECIES_TREE_DIR: &str = "species_tree";

pub struct MlSpeciesTree<'a> {
    pub alignments: &'a AlignmentFiles,
    pub iqtree_meta: &'a DepMetadata,
    pub iqtree_configs: &'a IqTreeConfig,
    pub output_dir: &'a Path,
    pub prefix: &'a str,
    pub enforce_v1: bool,
}

impl<'a> MlSpeciesTree<'a> {
    /// Create a new instance of `MlIqTree`
    pub fn new(
        alignments: &'a AlignmentFiles,
        iqtree_meta: &'a DepMetadata,
        iqtree_configs: &'a IqTreeConfig,
        output_dir: &'a Path,
        prefix: &'a str,
    ) -> Self {
        Self {
            alignments,
            iqtree_meta,
            iqtree_configs,
            output_dir,
            prefix,
            enforce_v1: false,
        }
    }

    pub fn enforce_v1(mut self, enforce_v1: bool) -> Self {
        self.enforce_v1 = enforce_v1;
        self
    }

    pub fn infer(&self, prefix: &str) {
        let output_dir = self.output_dir.join(SPECIES_TREE_DIR);
        PathCheck::new(&output_dir).is_dir().prompt_exists(false);
        let spinner = common::init_spinner();
        spinner.set_message("Concatenating alignments");
        let (alignment_path, partition_path) = self.concat_alignments(&output_dir);
        let spinner_msg = format!(
            "Running IQ-TREE. Check the IQ-TREE log for details: {}",
            output_dir.join(prefix).with_extension("log").display()
        );
        spinner.set_message(spinner_msg);
        let output_path = output_dir.join(prefix);
        let iqtree = IqTree::new(self.iqtree_configs, self.iqtree_meta);
        let out = iqtree.run_species_tree(&alignment_path, &partition_path, &output_path);
        spinner.finish_with_message("IQ-TREE finished");
        if !out.status.success() {
            log::error!("IQ-TREE failed to run: {:?}", iqtree);
            return;
        }
        log::info!("IQ-TREE finished successfully.");
    }

    fn concat_alignments(&self, output_dir: &Path) -> (PathBuf, PathBuf) {
        let output_pre = Path::new(self.prefix);
        let input_fmt = InputFmt::Auto;
        let output_fmt = OutputFmt::Phylip;
        let partition_fmt = PartitionFmt::Raxml;
        let datatype = DataType::Dna;
        let mut alignment_files = self
            .alignments
            .files
            .iter()
            .map(|f| f.parent_dir.join(&f.file_name))
            .collect::<Vec<PathBuf>>();
        let output_path = files::create_output_fname(output_dir, output_pre, &output_fmt);
        let mut concat = Concat::new(&mut alignment_files, &input_fmt, &datatype);
        concat.concat_alignment_no_spinner();
        let mut writer = SeqWriter::new(&output_path, &concat.alignment, &concat.header);
        writer
            .write_sequence(&output_fmt)
            .expect("Failed writing the output file");
        let partition_path = output_dir.join("partition").with_extension("txt");
        let part_writer = PartWriter::new(
            &partition_path,
            &concat.partition,
            &partition_fmt,
            &datatype,
        );
        part_writer.write_partition();
        (output_path, partition_path)
    }
}

pub struct MLGeneTree<'a> {
    pub alignment: &'a Path,
    pub iqtree_meta: &'a DepMetadata,
    pub output_dir: &'a Path,
    pub prefix: &'a str,
    pub enforce_v1: bool,
}

impl<'a> MLGeneTree<'a> {
    pub fn new(
        alignment: &'a Path,
        iqtree_meta: &'a DepMetadata,
        output_dir: &'a Path,
        prefix: &'a str,
        enforce_v1: bool,
    ) -> Self {
        Self {
            alignment,
            iqtree_meta,
            output_dir,
            prefix,
            enforce_v1,
        }
    }
}

#[derive(Debug)]
struct IqTree<'a> {
    configs: &'a IqTreeConfig,
    metadata: &'a DepMetadata,
}

impl<'a> IqTree<'a> {
    fn new(configs: &'a IqTreeConfig, metadata: &'a DepMetadata) -> Self {
        Self { configs, metadata }
    }

    fn run_species_tree(&self, alignment: &Path, partition: &Path, output_path: &Path) -> Output {
        let executable = match &self.metadata.executable {
            Some(e) => e,
            None => IQTREE2_EXE,
        };
        let mut out = Command::new(executable);
        out.arg("-s")
            .arg(alignment)
            .arg("-q")
            .arg(partition)
            .arg("-m")
            .arg(&self.configs.models)
            .arg("--prefix")
            .arg(output_path)
            .arg("-B")
            .arg(&self.configs.bootstrap)
            .arg("-T")
            .arg(&self.configs.threads);

        // if !other_args.is_empty() {
        //     parse_override_args!(out, other_args);
        // }

        out.output().expect("Failed to run IQ-TREE")
    }

    // fn get_args(&mut self, executable: Option<&str>) {
    //     match executable {
    //         Some(e) => self.get_executable(e),
    //         None => self.get_executable(IQTREE2_EXE),
    //     };
    //     self.models = self.capture_models();
    //     self.threads = self.capture_threads();
    //     self.bootstrap = self.capture_bs_value();
    // }

    // fn get_executable(&mut self, executable: &str) {
    //     if self.enforce_v1 {
    //         self.executable = IQTREE_EXE.to_string();
    //     } else {
    //         self.executable = executable.to_string();
    //     }
    // }

    // fn capture_models(&mut self) -> String {
    //     let re = Regex::new(r"(?<models>-m)\s+(?<value>\S+)").expect("Failed to compile regex");
    //     let capture = re.captures(&self.args).expect("Failed to capture models");
    //     match capture.name("value") {
    //         Some(v) => {
    //             let value = v.as_str().to_string();
    //             let model = format!("{} {}", capture.name("models").unwrap().as_str(), value);
    //             self.args = self.args.replace(&model, "");
    //             value
    //         }
    //         None => DEFAULT_MODELS.to_string(),
    //     }
    // }

    // fn capture_bs_value(&mut self) -> String {
    //     let re = Regex::new(r"(?<bs>-B|b)\s+(?<value>\d+)").expect("Failed to compile regex");
    //     let bs = re
    //         .captures(&self.args)
    //         .expect("Failed to capture bootstrap value");
    //     match bs.name("value") {
    //         Some(v) => {
    //             let value = v.as_str().to_string();
    //             let arg = format!("{} {}", bs.name("bs").unwrap().as_str(), value);
    //             self.args = self.args.replace(&arg, "");
    //             value
    //         }
    //         None => DEFAULT_BOOTSTRAP.to_string(),
    //     }
    // }

    // fn capture_threads(&mut self) -> String {
    //     let re = Regex::new(r"(?<threads>-T|t)\s+(?<value>\d+)").expect("Failed to compile regex");
    //     let thread = re
    //         .captures(&self.args)
    //         .expect("Failed to capture thread value");
    //     match thread.name("value") {
    //         Some(v) => {
    //             let value = v.as_str().to_string();
    //             let arg = format!("{} {}", thread.name("threads").unwrap().as_str(), value);
    //             self.args = self.args.replace(&arg, "");
    //             value
    //         }
    //         None => DEFAULT_THREADS.to_string(),
    //     }
    // }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     macro_rules! init {
//         ($iqtree: ident) => {
//             let mut $iqtree = IqTree::new(None, false);
//         };
//     }

//     #[test]
//     fn test_bootstrap_value() {
//         init!(iqtree);
//         let bs = iqtree.capture_bs_value();
//         assert_eq!(bs, "1000");
//     }

//     #[test]
//     fn test_threads_value() {
//         init!(iqtree);
//         let threads = iqtree.capture_threads();
//         assert_eq!(threads, "4");
//     }
// }
