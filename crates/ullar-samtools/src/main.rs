use std::path::PathBuf;

use clap::{Args, Parser, builder, crate_authors, crate_description, crate_name, crate_version};
use ullar_samtools::{
    batch::{index::BatchIndexBams, phase::BatchPhaseBam},
    samtools::{faidx::SamtoolsFaIndex, sort::SamtoolsSort},
    types::SamtoolsIndexFormat,
};

fn main() {
    let cli = Cli::parse();
    ullar_logger::init_logger(std::path::Path::new("ullar-gatk.log"))
        .expect("Failed to initialize logger");
    match cli {
        Cli::Sort(sort_args) => run_sort(sort_args).expect("Failed to run sort"),
        Cli::Faidx(faidx_args) => run_faidx(faidx_args).expect("Failed to run faidx"),
        Cli::Phase(phase_args) => run_phase(phase_args).expect("Failed to run phase"),
        Cli::Index(index_args) => run_index(index_args).expect("Failed to run index"),
    }
}

#[derive(Parser)]
#[command(name = crate_name!(), version = crate_version!(), about = crate_description!(), author = crate_authors!())]
enum Cli {
    #[command(name = "sort", about = "Sort a BAM file using GATK")]
    Sort(Sort),
    #[command(
        name = "faidx",
        about = "Index a reference fasta file using samtools faidx"
    )]
    Faidx(FaidxArgs),
    #[command(
        name = "index",
        about = "Index BAM files in a directory using samtools index"
    )]
    Index(IndexArgs),
    #[command(
        name = "phase",
        about = "Phase reads in a BAM file using samtools phase"
    )]
    Phase(PhaseArgs),
}

#[derive(Args)]
struct Sort {
    #[arg(short, long, help = "Path to the input BAM/CRAM file")]
    input: String,
    #[arg(short, long, help = "Path to the output BAM/CRAM file")]
    output: Option<String>,
    #[arg(
        short,
        long,
        help = "Sort order (e.g., 'coordinate', 'queryname')",
        default_value = "coordinate",
        value_parser = builder::PossibleValuesParser::new(["coordinate", "queryname", "duplicate", "unsorted"])
    )]
    sort_order: String,
    #[arg(long, help = "Create index for the output file")]
    create_index: bool,
    #[arg(long, help = "Temporary directory for intermediate files")]
    temp_dir: Option<String>,
    #[arg(long, help = "Path to the GATK executable")]
    exe: Option<String>,
}

#[derive(Args)]
struct FaidxArgs {
    #[arg(short, long, help = "Path to the reference fasta file")]
    reference: String,
    #[arg(short, long, help = "Path to the output index file (optional)")]
    output: Option<String>,
    #[arg(
        short,
        long,
        help = "Output format of the index file (fai or fai.gz)",
        default_value = "fai",
        value_parser = builder::PossibleValuesParser::new(["fai", "fai.gz"])
    )]
    format: String,
    #[arg(
        long,
        help = "Additional optional arguments for samtools faidx command"
    )]
    optional_args: Vec<String>,
}

#[derive(Args)]
struct IndexArgs {
    #[arg(short, long, help = "Path to the BAM file directory")]
    dir: String,
    #[arg(long, help = "Find reads recursively in the read directory")]
    recursive: bool,
    #[arg(long, help = "Dry run mode (do not execute commands)")]
    dry_run: bool,
}

#[derive(Args)]
struct PhaseArgs {
    #[arg(short, long, help = "Path to the BAM file directory")]
    dir: String,
    #[arg(short, long, help = "Path to the output BAM file directory")]
    output: String,
    #[arg(short, long, help = "Path to the reference fasta file directory")]
    reference: String,
    #[arg(long, help = "Dry run mode (do not execute commands)")]
    dry_run: bool,
    #[arg(long, help = "Find reads recursively in the read directory")]
    recursive: bool,
    // #[arg(long, help = "Drop reads with ambiguous phase")]
    // drop_ambiguous: bool,
    // #[arg(long, help = "Skip chimera check")]
    // skip_chimera_check: bool,
    // #[arg(long, help = "Maximum length for local phasing")]
    // max_phase_length: Option<usize>,
    // #[arg(
    //     long,
    //     help = "Minimum Phred quality for a base to be considered in phasing"
    // )]
    // min_base_quality: Option<u8>,
    // #[arg(
    //     long,
    //     help = "Additional optional arguments for samtools phase command"
    // )]
    // optional_args: Vec<String>,
}

fn run_faidx(args: FaidxArgs) -> Result<(), Box<dyn std::error::Error>> {
    let mut faidx = SamtoolsFaIndex::new(&args.reference);

    if let Some(output) = args.output {
        faidx.output_path(output);
    }

    let format = args.format.parse::<SamtoolsIndexFormat>()?;
    faidx.output_format(format);

    if !args.optional_args.is_empty() {
        faidx.add_optional_arg(&args.optional_args);
    }

    faidx.create_index()?;
    Ok(())
}

fn run_sort(args: Sort) -> Result<(), Box<dyn std::error::Error>> {
    let mut sort = SamtoolsSort::new(&args.input);

    match args.output {
        Some(output) => {
            sort.output_path(output);
        }
        None => {
            let default_output = PathBuf::from(&args.input).with_file_name(format!(
                "{}_sorted.bam",
                PathBuf::from(&args.input)
                    .file_stem()
                    .unwrap()
                    .to_string_lossy()
            ));
            sort.output_path(&default_output);
        }
    }
    Ok(())
}

fn run_phase(args: PhaseArgs) -> Result<(), Box<dyn std::error::Error>> {
    let mut phase = BatchPhaseBam::new(&args.dir);

    phase
        .output_dir(&args.output)
        .reference_dir(&args.reference)
        .recursive(args.recursive);

    if args.dry_run {
        phase.dry_run();
    }
    phase.phase()?;
    Ok(())
}

fn run_index(args: IndexArgs) -> Result<(), Box<dyn std::error::Error>> {
    let mut indexer = BatchIndexBams::new(&args.dir);
    indexer.recursive(args.recursive);

    if args.dry_run {
        indexer.dry_run();
    }
    indexer.index();
    Ok(())
}
