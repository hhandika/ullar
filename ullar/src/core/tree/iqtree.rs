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

use super::configs::IqTreeParams;

const SPECIES_TREE_DIR: &str = "species_tree";

pub struct MlSpeciesTree<'a> {
    pub alignments: &'a AlignmentFiles,
    pub iqtree_configs: &'a IqTreeParams,
    pub output_dir: &'a Path,
    pub prefix: &'a str,
    pub enforce_v1: bool,
}

impl<'a> MlSpeciesTree<'a> {
    /// Create a new instance of `MlIqTree`
    pub fn new(
        alignments: &'a AlignmentFiles,
        iqtree_configs: &'a IqTreeParams,
        output_dir: &'a Path,
        prefix: &'a str,
    ) -> Self {
        Self {
            alignments,
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
        let meta = match &self.iqtree_configs.dependency {
            Some(m) => m,
            None => {
                log::error!(
                    "IQ-TREE dependency not found in the config.\
                Check IQ-TREE installation and config files."
                );
                return;
            }
        };
        let iqtree = IqTree::new(self.iqtree_configs, &meta);
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
    configs: &'a IqTreeParams,
    metadata: &'a DepMetadata,
}

impl<'a> IqTree<'a> {
    fn new(configs: &'a IqTreeParams, metadata: &'a DepMetadata) -> Self {
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
            .arg(self.get_bootstrap_species())
            .arg("-T")
            .arg(&self.configs.threads);

        // if !other_args.is_empty() {
        //     parse_override_args!(out, other_args);
        // }

        out.output().expect("Failed to run IQ-TREE")
    }

    fn get_bootstrap_species(&self) -> String {
        match &self.configs.bootstrap {
            Some(bs) => bs.to_string(),
            None => "1000".to_string(),
        }
    }
}
