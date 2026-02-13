use std::path::Path;

use clap::{Args, Parser, builder, crate_authors, crate_description, crate_name, crate_version};

use ullar_bwa::{
    batch::{multi_refs::BatchBwaAlignMultiRefs, single_ref::BatchBwaAlignSingleRef},
    bwa::{index::BwaIndex, mem::BwaMem, metadata::BwaMetadata, types::BwaFormat},
};

fn main() {
    let cli = Cli::parse();
    ullar_logger::init_logger(Path::new("ullar-bwa.log")).expect("Failed to initialize logger");
    match cli {
        Cli::Index(index_args) => run_index(index_args),
        Cli::Align(align_args) => run_align(align_args),
        Cli::BatchAlign(batch_args) => run_batch_align(batch_args),
        Cli::BatchSampleAlign(batch_sample_args) => run_batch_sample_align(batch_sample_args),
        Cli::Deps => check_bwa_installed(),
    }
}

#[derive(Parser)]
#[command(name = crate_name!(), version = crate_version!(), about = crate_description!(), author = crate_authors!())]
enum Cli {
    #[command(name = "index", about = "Index a reference genome using BWA")]
    Index(Index),
    #[command(name = "align", about = "Align reads to a reference genome using BWA")]
    Align(Align),
    #[command(
        name = "batch",
        about = "Perform batch BWA alignment on a directory of reads"
    )]
    BatchAlign(BatchAlign),
    #[command(
        name = "batch-sample",
        about = "Perform batch BWA alignment on sample/individual-specific references"
    )]
    BatchSampleAlign(BatchSampleAlign),
    #[command(name = "deps", about = "Print help information")]
    Deps,
}

#[derive(Args)]
struct Index {
    #[arg(short, long, help = "Path to the reference file")]
    reference: String,
    #[arg(
        short,
        long,
        help = "Prefix for the index files. If not provided, defaults to the reference file name"
    )]
    index_prefix: Option<String>,
    #[arg(
        short,
        long,
        help = "Algorithm to use for indexing",
        default_value = "is"
    )]
    algorithm: String,
    #[arg(
        short,
        long,
        help = "BWA executable to use",
        default_value = "bwa-mem2",
        value_parser = builder::PossibleValuesParser::new(["bwa", "bwa-mem2", "bwa-mem2.avx", "bwa-mem2.avx2", "bwa-mem2.avx512bw", "bwa-mem2.sse41", "bwa-mem2.sse42"])
    )]
    executable: String,
}

#[derive(Args)]
struct Align {
    #[arg(short, long, help = "Path to the reference file")]
    reference: String,
    #[arg(long, help = "Path to the query read 1 file")]
    read1: String,
    #[arg(long, help = "Path to the query read 2 file")]
    read2: Option<String>,
    #[arg(short, long, help = "Sample name for the alignment")]
    sample_name: String,
    #[arg(
        short = 'F',
        long,
        help = "Output format",
        default_value = "bam",
        value_parser = builder::PossibleValuesParser::new(["sam", "bam"])
    )]
    output_format: String,
    #[command(flatten)]
    common: CommonArgs,
}

#[derive(Args)]
struct BatchAlign {
    #[arg(short, long, help = "Path to the directory containing reads")]
    dir: String,
    #[arg(long, help = "Recursively search for reads in subdirectories")]
    recursive: bool,
    #[arg(short, long, help = "Number of threads to use", default_value_t = 4)]
    threads: usize,
    #[arg(long, help = "Test mode: only list found samples without aligning")]
    dry_run: bool,
    #[command(flatten)]
    common: CommonArgs,
}

#[derive(Args)]
struct BatchSampleAlign {
    #[arg(short, long, help = "Path to the directory containing reads")]
    dir: String,
    #[arg(short, long, help = "Path to the reference directory")]
    reference_dir: String,
    #[arg(long, help = "Recursively search for reads in subdirectories")]
    recursive: bool,
    #[arg(short, long, help = "Number of threads to use", default_value_t = 4)]
    threads: usize,
    #[arg(long, help = "Test mode: only list found samples without aligning")]
    dry_run: bool,
    #[arg(short, long, help = "Path to the output directory")]
    output: String,
    #[arg(
        short,
        long,
        help = "BWA executable to use",
        default_value = "bwa-mem2",
        value_parser = builder::PossibleValuesParser::new(["bwa", "bwa-mem2", "bwa-mem2.avx", "bwa-mem2.avx2", "bwa-mem2.avx512bw", "bwa-mem2.sse41", "bwa-mem2.sse42"])
    )]
    executable: String,
}

#[derive(Args)]
struct CommonArgs {
    #[arg(short, long, help = "Path to the reference file")]
    reference: String,
    #[arg(short, long, help = "Path to the output directory")]
    output: String,
    #[arg(
        short,
        long,
        help = "BWA executable to use",
        default_value = "bwa-mem2",
        value_parser = builder::PossibleValuesParser::new(["bwa", "bwa-mem2", "bwa-mem2.avx", "bwa-mem2.avx2", "bwa-mem2.avx512bw", "bwa-mem2.sse41", "bwa-mem2.sse42"])
    )]
    executable: String,
}

fn run_index(args: Index) {
    let mut bwa_index = BwaIndex::new(&args.reference);
    bwa_index.algorithm(args.algorithm);
    if let Some(prefix) = args.index_prefix {
        bwa_index.index_prefix(&prefix);
    }
    let exe = args.executable.parse().unwrap_or_default();
    bwa_index.set_executable(exe).index();
}

fn run_align(args: Align) {
    let mut bwa_mem = BwaMem::new(&args.sample_name);
    let output_fmt = args
        .output_format
        .to_lowercase()
        .parse()
        .unwrap_or(BwaFormat::Bam);
    bwa_mem
        .reference_path(&args.reference)
        .query_read1(&args.read1)
        .output_format(output_fmt)
        .query_read2(args.read2)
        .output_path(&args.common.output);
    bwa_mem.align().expect("Failed to run BWA mem");
}

fn run_batch_align(args: BatchAlign) {
    let batch = BatchBwaAlignSingleRef::new(&args.dir)
        .reference(&args.common.reference)
        .output(&args.common.output)
        .recursive(args.recursive)
        .threads(args.threads)
        .bwa_executable(&args.common.executable);
    if args.dry_run {
        batch.dry_run();
    } else {
        batch.run().expect("Failed to run batch BWA alignment");
    }
}

fn run_batch_sample_align(args: BatchSampleAlign) {
    let mut batch = BatchBwaAlignMultiRefs::new(&args.dir);
    batch
        .reference_dir(&args.reference_dir)
        .output_dir(&args.output)
        .recursive(args.recursive)
        .bwa_executable(&args.executable);

    batch
        .run()
        .expect("Failed to run batch BWA alignment with sample-specific references");
}

fn check_bwa_installed() {
    let mut meta = BwaMetadata::new();
    meta.get();

    match meta.version {
        Some(version) => {
            println!("BWA is installed. Version: {}", version);
        }
        None => {
            println!("BWA is not installed or not found in PATH.");
        }
    }
}
