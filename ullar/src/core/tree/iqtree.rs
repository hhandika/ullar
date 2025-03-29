//! Species and gene tree inference using IQ-TREE.
use core::str;
use std::{
    fs::{create_dir_all, File},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    process::{Command, Output},
};

use rayon::prelude::*;
use segul::{
    helper::{
        concat::Concat,
        files,
        types::{DataType, InputFmt, OutputFmt, PartitionFmt},
    },
    writer::{partition::PartWriter, sequences::SeqWriter},
};

use crate::{
    core::deps::{
        iqtree::{IqTreeParams, IQTREE2_EXE, IQTREE_EXE},
        DepMetadata,
    },
    helper::common,
    parse_override_args,
    types::{alignments::AlignmentFiles, trees::IQTreePartitions},
};

const GENE_TREE_FILENAME: &str = "genes";
const MULTI_TREE_EXTENSION: &str = "trees";
const TREE_FILE_EXTENSION: &str = "treefile";
const SPECIES_TREE_BEST_MODEL_EXTENSION: &str = "best_model.nex";

pub struct MlSpeciesTree<'a> {
    pub alignments: &'a AlignmentFiles,
    pub iqtree_configs: &'a IqTreeParams,
    pub output_dir: &'a Path,
    pub codon_model: bool,
}

impl<'a> MlSpeciesTree<'a> {
    /// Create a new instance of `MlIqTree`
    pub fn new(
        alignments: &'a AlignmentFiles,
        iqtree_configs: &'a IqTreeParams,
        output_dir: &'a Path,
        codon_model: bool,
    ) -> Self {
        Self {
            alignments,
            iqtree_configs,
            output_dir,
            codon_model,
        }
    }

    pub fn infer_species_tree(
        &self,
        iqtree_result: &mut IQTreeResults,
        prefix: &'a str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let spinner = common::init_spinner();
        spinner.set_message("Concatenating alignments");
        let (alignment_path, partition_path) = self.concat_alignments(&self.output_dir, prefix);
        spinner.set_message("Running IQ-TREE for species tree");
        let output_dir = self.output_dir.join(prefix);
        let meta = match &self.iqtree_configs.dependency {
            Some(m) => m,
            None => {
                log::error!(
                    "IQ-TREE dependency not found in the config.\
                Check IQ-TREE installation and config files."
                );
                return Err("IQ-TREE dependency not found".into());
            }
        };
        let iqtree = IqTree::new(self.iqtree_configs, &meta);
        let out = iqtree.infer_species_tree(&alignment_path, &partition_path, &output_dir);
        spinner.finish_with_message("IQ-TREE finished estimating the species tree\n");
        if !out.status.success() {
            return Err("IQ-TREE failed to run".into());
        }
        let tree_path = output_dir.with_extension(TREE_FILE_EXTENSION);
        if !tree_path.exists() {
            return Err("Species tree file not found. Check IQ-TREE output \
                 if it ran successfully."
                .into());
        }
        let best_model_path = output_dir.with_extension(SPECIES_TREE_BEST_MODEL_EXTENSION);
        iqtree_result.add_alignment(alignment_path);
        iqtree_result.add_partition(partition_path);
        iqtree_result.add_species_tree(tree_path);
        iqtree_result.add_species_tree_model(best_model_path);
        Ok(())
    }

    fn concat_alignments(&self, output_dir: &Path, prefix: &str) -> (PathBuf, PathBuf) {
        let output_pre = Path::new(prefix);
        let input_fmt = InputFmt::Auto;
        let output_fmt = OutputFmt::Phylip;
        let partition_fmt = if self.codon_model {
            PartitionFmt::RaxmlCodon
        } else {
            PartitionFmt::Raxml
        };
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

pub struct MlGeneTree<'a> {
    pub alignments: &'a AlignmentFiles,
    pub iqtree_configs: &'a IqTreeParams,
    pub output_dir: &'a Path,
}

impl<'a> MlGeneTree<'a> {
    pub fn new(
        alignments: &'a AlignmentFiles,
        iqtree_configs: &'a IqTreeParams,
        output_dir: &'a Path,
    ) -> Self {
        Self {
            alignments,
            iqtree_configs,
            output_dir,
        }
    }

    pub fn infer_gene_trees(&self, iqtree_result: &mut IQTreeResults) {
        let progress_bar = common::init_progress_bar(self.alignments.file_counts as u64);
        log::info!("Running IQ-TREE for gene trees");
        progress_bar.set_message("gene trees");
        self.alignments.files.par_iter().for_each(|f| {
            let alignment_path = f.parent_dir.join(&f.file_name);
            let file_stem = alignment_path.file_stem().expect("Failed to get file stem");
            let output_dir = self.output_dir.join(file_stem);
            create_dir_all(&output_dir).expect("Failed to create output directory");
            let full_path = output_dir.join(file_stem);
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
            let out = iqtree.infer_gene_trees(&alignment_path, &full_path);
            if !out.status.success() {
                let error = str::from_utf8(&out.stderr).expect("Failed to read error message");
                let message = format!(
                    "Failed to run IQ-TREE for gene {}: {}",
                    file_stem.to_string_lossy(),
                    error
                );
                log::error!("{}", message);
                return;
            }
            progress_bar.inc(1);
        });
        progress_bar.finish_with_message("gene trees\n");
        let gene_trees = self.find_gene_trees(&self.output_dir);
        let gene_tree_path = self.combine_gene_trees(&gene_trees);
        iqtree_result.add_gene_trees(gene_tree_path);
    }

    fn find_gene_trees(&self, output_dir: &Path) -> Vec<PathBuf> {
        let pattern = format!("{}/*/*.treefile", output_dir.display());
        let gene_trees = glob::glob(&pattern)
            .expect("Failed to find gene trees")
            .filter_map(|f| f.ok())
            .collect::<Vec<PathBuf>>();
        gene_trees
    }

    fn combine_gene_trees(&self, gene_trees: &[PathBuf]) -> PathBuf {
        let output_path = self
            .output_dir
            .join(GENE_TREE_FILENAME)
            .with_extension(MULTI_TREE_EXTENSION);
        let file = File::create(&output_path).expect("Failed to create file");
        let mut buff = BufWriter::new(&file);
        for tree in gene_trees {
            let content = std::fs::read_to_string(tree).expect("Failed to read file");
            writeln!(buff, "{}", content.trim()).expect("Failed to write to file");
        }
        buff.flush().expect("Failed to flush buffer");
        output_path
    }
}

pub struct GeneSiteConcordance<'a> {
    pub iqtree_configs: &'a IqTreeParams,
    pub output_dir: &'a Path,
}

impl<'a> GeneSiteConcordance<'a> {
    pub fn new(iqtree_configs: &'a IqTreeParams, output_dir: &'a Path) -> Self {
        Self {
            iqtree_configs,
            output_dir,
        }
    }

    pub fn infer_concordance_factor(
        &self,
        iqtree_result: &IQTreeResults,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let invalid_result =
            !iqtree_result.concatenated_alignment.exists() && !iqtree_result.species_tree.exists();
        if invalid_result {
            return Err("IQ-TREE errors, missing data".into());
        }
        let spinner = common::init_spinner();
        spinner.set_message("Running IQ-TREE for gene concordance factor");
        create_dir_all(&self.output_dir).expect("Failed to create output directory");

        let meta = match &self.iqtree_configs.dependency {
            Some(m) => m,
            None => {
                log::error!(
                    "IQ-TREE dependency not found in the config.\
                Check IQ-TREE installation and config files."
                );
                return Err("IQ-TREE dependency not found".into());
            }
        };
        let prefix_gene = "gene_concordance";
        let output_gene = self.output_dir.join(prefix_gene);
        let iqtree = IqTree::new(self.iqtree_configs, &meta);
        let out_gene = iqtree.infer_gene_concordance(&iqtree_result, &output_gene);
        spinner.set_message("Running IQ-TREE for site concordance factor");
        let prefix_site = "site_concordance";
        let output_site = self.output_dir.join(prefix_site);
        let out_site = iqtree.infer_site_concordance(&iqtree_result, &output_site);
        let mut error = String::new();

        if !out_gene.status.success() {
            if !out_gene.status.success() {
                let err = str::from_utf8(&out_gene.stderr).expect("Failed to read error message");
                let message = format!("Failed to run IQ-TREE for gene concordance: {}\n\n", err);
                error.push_str(&message);
            }
        }

        if !out_site.status.success() {
            let err = str::from_utf8(&out_site.stderr).expect("Failed to read error message");
            let message = format!("Failed to run IQ-TREE for site concordance: {}\n", err);
            error.push_str(&message);
        }
        spinner.finish_with_message("IQ-TREE finished estimating concordance factors\n");
        Ok(())
    }
}

pub struct IQTreeResults {
    pub concatenated_alignment: PathBuf,
    pub partition: PathBuf,
    pub species_tree: PathBuf,
    pub gene_trees: PathBuf,
    pub species_tree_best_model: PathBuf,
}

impl IQTreeResults {
    pub fn new() -> Self {
        Self {
            species_tree: PathBuf::new(),
            gene_trees: PathBuf::new(),
            concatenated_alignment: PathBuf::new(),
            partition: PathBuf::new(),
            species_tree_best_model: PathBuf::new(),
        }
    }

    pub fn add_alignment(&mut self, path: PathBuf) {
        self.concatenated_alignment = path;
    }

    pub fn add_partition(&mut self, path: PathBuf) {
        self.partition = path;
    }

    pub fn add_species_tree(&mut self, path: PathBuf) {
        self.species_tree = path;
    }

    pub fn add_gene_trees(&mut self, path: PathBuf) {
        self.gene_trees = path;
    }

    pub fn add_species_tree_model(&mut self, path: PathBuf) {
        self.species_tree_best_model = path;
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

    fn infer_species_tree(&self, alignment: &Path, partition: &Path, output_path: &Path) -> Output {
        let executable = self.get_executable();
        let mut out = Command::new(executable);
        out.arg("-s")
            .arg(alignment)
            .arg(&self.get_partition_arg())
            .arg(partition)
            .arg("-m")
            .arg(&self.configs.models)
            .arg("--prefix")
            .arg(output_path)
            .arg("-B")
            .arg(self.get_bootstrap_species())
            .arg("-T")
            .arg(&self.configs.threads);

        if let Some(opt_args) = &self.configs.optional_args {
            parse_override_args!(out, opt_args);
        }

        out.output().expect("Failed to run IQ-TREE")
    }

    fn get_partition_arg(&self) -> String {
        match &self.configs.partition_model {
            Some(p) => p.get_arg(),
            None => IQTreePartitions::default().get_arg(),
        }
    }

    fn infer_gene_trees(&self, alignment: &Path, full_path: &Path) -> Output {
        let executable = self.get_executable();
        let mut out = Command::new(executable);

        out.arg("-s")
            .arg(&alignment)
            .arg("-m")
            .arg(&self.configs.models)
            .arg("--prefix")
            .arg(full_path)
            .arg("-T")
            .arg("1");
        if let Some(opt_args) = &self.configs.optional_args {
            parse_override_args!(out, opt_args);
        }

        out.output().expect("Failed to run IQ-TREE")
    }

    fn infer_gene_concordance(&self, iqtree_result: &IQTreeResults, output_path: &Path) -> Output {
        let executable = self.get_any_executable();
        let mut out = Command::new(executable);
        out.arg("-t")
            .arg(&iqtree_result.species_tree)
            .arg("--gcf")
            .arg(&iqtree_result.gene_trees)
            .arg("--prefix")
            .arg(output_path);

        out.output()
            .expect("Failed to run site concordance analyses.")
    }

    fn infer_site_concordance(&self, iqtree_result: &IQTreeResults, output_path: &Path) -> Output {
        let executable = self.get_any_executable();
        let mut out = Command::new(executable);
        out.arg("-te")
            .arg(&iqtree_result.species_tree)
            .arg("-s")
            .arg(&iqtree_result.concatenated_alignment)
            .arg("--prefix")
            .arg(output_path);

        if let Some(opt_args) = &self.configs.optional_args {
            parse_override_args!(out, opt_args);
        } else {
            out.arg("-blfix").arg("--scfl").arg("1000");
        }
        out.output()
            .expect("Failed to run site concordance analyses.")
    }

    fn get_executable(&self) -> String {
        if self.configs.force_v1 {
            return IQTREE_EXE.to_string();
        }
        self.get_any_executable()
    }

    fn get_any_executable(&self) -> String {
        match &self.metadata.executable {
            Some(e) => e.to_string(),
            None => IQTREE2_EXE.to_string(),
        }
    }

    fn get_bootstrap_species(&self) -> String {
        match &self.configs.bootstrap {
            Some(bs) => bs.to_string(),
            None => "1000".to_string(),
        }
    }
}
