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

use crate::{core::deps::DepMetadata, types::alignments::AlignmentFiles};

/// Default parameters for IQ-TREE 2
pub const DEFAULT_IQTREE2_PARAMS: &str = "-m GTR+G -B 1000";

/// Default parameters for IQ-TREE 1
pub const DEFAULT_IQTREE1_PARAMS: &str = "-m GTR+G -bb 1000";

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
        let mut out = Command::new(self.get_executable());
        out.arg("-s")
            .arg(alignment)
            .arg("-q")
            .arg(partition)
            .arg("-m")
            .arg("GTR+G+I")
            .arg("--prefix")
            .arg(output_path)
            .arg("-B")
            .arg("1000")
            .arg("-T")
            .arg("4")
            .arg("-bnni");

        out.output().expect("Failed to run IQ-TREE")
    }

    fn get_executable(&self) -> &str {
        match self.iqtree_meta {
            Some(meta) => meta.executable.as_str(),
            None => "iqtree2",
        }
    }
}
