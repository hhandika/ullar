use clap::{Args, Parser, builder};
use ullar_bwa::{
    batch::BatchBwaAlign,
    bwa::{index::BwaIndex, mem::BwaMem},
};

fn main() {
    let cli = Cli::parse();

    match cli {
        Cli::Index(index_args) => run_index(index_args),
        Cli::Align(align_args) => run_align(align_args),
        Cli::BatchAlign(batch_args) => run_batch_align(batch_args),
    }
}

#[derive(Parser)]
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
}

#[derive(Args)]
struct Align {
    #[arg(short, long, help = "Path to the reference file")]
    reference: String,
    #[arg(long, help = "Path to the query read 1 file")]
    read1: String,
    #[arg(long, help = "Path to the query read 2 file")]
    read2: Option<String>,
    #[arg(short = 'F', long, help = "Output format", default_value = "bam", value_parser = builder::PossibleValuesParser::new(["sam", "bam"]))]
    output_format: String,
    #[arg(short, long, help = "Path to the output file")]
    output: String,
}

#[derive(Args)]
struct BatchAlign {
    #[arg(short, long, help = "Path to the directory containing reads")]
    dir: String,
    #[arg(short, long, help = "Path to the reference file")]
    reference: String,
    #[arg(short, long, help = "Path to the output directory")]
    output: String,
    #[arg(long, help = "Recursively search for reads in subdirectories")]
    recursive: bool,
    #[arg(short, long, help = "Number of threads to use", default_value_t = 4)]
    threads: usize,
    #[arg(long, help = "Test mode: only list found samples without aligning")]
    dry_run: bool,
}

fn run_index(args: Index) {
    let bwa = BwaIndex::build()
        .reference_path(&args.reference)
        .index_prefix(args.index_prefix)
        .algorithm(&args.algorithm)
        .build()
        .expect("Failed to build BWA index");

    bwa.index();
}

fn run_align(args: Align) {
    let bwa_mem = BwaMem::builder()
        .reference_path(&args.reference)
        .query_read1(&args.read1)
        .output_format(&args.output_format)
        .query_read2(args.read2)
        .output_path(&args.output)
        .build()
        .expect("Failed to build BWA mem");

    bwa_mem.align().expect("Failed to run BWA mem");
}

fn run_batch_align(args: BatchAlign) {
    let batch = BatchBwaAlign::builder()
        .dir(&args.dir)
        .reference(&args.reference)
        .output(&args.output)
        .recursive(args.recursive)
        .threads(args.threads)
        .build()
        .expect("Failed to build Batch BWA Align");
    if args.dry_run {
        batch.dry_run();
    } else {
        batch.run();
    }
}
