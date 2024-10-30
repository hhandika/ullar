//! Species and gene tree inference using IQ-TREE.
use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
};

use regex::Regex;
use segul::{
    helper::{
        concat::Concat,
        files,
        types::{DataType, InputFmt, OutputFmt, PartitionFmt},
    },
    writer::{partition::PartWriter, sequences::SeqWriter},
};

use crate::{
    core::deps::{iqtree::IqtreeMetadata, DepMetadata},
    types::alignments::AlignmentFiles,
};

/// Default parameters for IQ-TREE 2
pub const DEFAULT_MODELS: &str = "-m GTR+G+I";

pub struct MlIqTree<'a> {
    pub alignments: &'a AlignmentFiles,
    pub iqtree_meta: Option<&'a DepMetadata>,
    pub output_dir: &'a Path,
    pub prefix: &'a str,
}

impl<'a> MlIqTree<'a> {
    /// Create a new instance of `MlIqTree`
    pub fn new(
        alignments: &'a AlignmentFiles,
        iqtree_meta: Option<&'a DepMetadata>,
        output_dir: &'a Path,
        prefix: &'a str,
    ) -> Self {
        Self {
            alignments,
            iqtree_meta,
            output_dir,
            prefix,
        }
    }

    pub fn infer(&self, prefix: &str) {
        let output_dir = self.output_dir.join("species_tree");
        let (alignment_path, partition_path) = self.concat_alignments(&output_dir);
        let output_path = output_dir.join(prefix);
        let iqtree = self.run_iqtree(&alignment_path, &partition_path, &output_path);
        // Check if the command was successful
        if !iqtree.status.success() {
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

    fn run_iqtree(&self, alignment: &Path, partition: &Path, output_path: &Path) -> Output {
        let iqtree = IqTree::new(self.iqtree_meta);
        let mut out = Command::new(iqtree.get_executable_name());
        out.arg("-s")
            .arg(alignment)
            .arg("-q")
            .arg(partition)
            .arg("-m")
            .arg(iqtree.get_models())
            .arg("--prefix")
            .arg(output_path)
            .arg("-B")
            .arg("1000")
            .arg("-T")
            .arg("10")
            .arg("-bnni");

        out.output().expect("Failed to run IQ-TREE")
    }
}

struct IqTree<'a> {
    executable: Option<&'a str>,
    override_args: Option<&'a str>,
}

impl<'a> IqTree<'a> {
    fn new(meta: Option<&'a DepMetadata>) -> Self {
        let executable = meta.map(|m| m.executable.as_str());
        let override_args = meta.and_then(|m| m.override_args.as_deref());
        Self {
            executable,
            override_args,
        }
    }

    fn get_executable_name(&self) -> String {
        match self.executable {
            Some(exe) => exe.to_string(),
            None => self.try_get_executable(),
        }
    }

    fn get_models(&self) -> String {
        match self.override_args {
            Some(args) => args.to_string(),
            None => DEFAULT_MODELS.to_string(),
        }
    }

    #[allow(dead_code)]
    fn capture_bs_value(&self, args: &str) -> String {
        let re = Regex::new(r"(?<bs>-B|b)\s+(?<value>\d+)").expect("Failed to compile regex");
        let bs = re
            .captures(args)
            .expect("Failed to capture bootstrap value");
        bs.name("value").unwrap().as_str().to_string()
    }

    #[allow(dead_code)]
    fn capture_threads(&self, args: &str) -> String {
        let re = Regex::new(r"(?<threads>-T|t)\s+(?<value>\d+)").expect("Failed to compile regex");
        let bs = re.captures(args).expect("Failed to capture thread value");
        bs.name("value").unwrap().as_str().to_string()
    }

    fn try_get_executable(&self) -> String {
        let meta = IqtreeMetadata::new(None);
        let dep = meta.get();
        match dep {
            Some(d) => d.executable,
            None => panic!("Failed to execute IQ-TREE. Please, check your IQ-TREE installation."),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! init {
        ($iqtree: ident) => {
            let $iqtree = IqTree {
                executable: None,
                override_args: None,
            };
        };
    }

    #[test]
    fn test_bootstrap_value() {
        let val = "-B 1000";
        init!(iqtree);
        let bs = iqtree.capture_bs_value(val);
        assert_eq!(bs, "1000");
    }

    #[test]
    fn test_threads_value() {
        let val = "-T 4";
        init!(iqtree);
        let threads = iqtree.capture_threads(val);
        assert_eq!(threads, "4");
    }
}
