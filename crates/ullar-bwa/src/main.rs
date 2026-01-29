use std::{fs, path::Path};

use clap::{Args, Parser, builder};
use log::LevelFilter;
use log4rs::{
    Config,
    append::{console::ConsoleAppender, file::FileAppender},
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
};

use ullar_bwa::{
    batch::BatchBwaAlign,
    bwa::{index::BwaIndex, mem::BwaMem, metadata::BwaMetadata},
};

fn main() {
    let cli = Cli::parse();
    init_logger(Path::new("ullar-bwa.log")).expect("Failed to initialize logger");
    match cli {
        Cli::Index(index_args) => run_index(index_args),
        Cli::Align(align_args) => run_align(align_args),
        Cli::BatchAlign(batch_args) => run_batch_align(batch_args),
        Cli::Deps => check_bwa_installed(),
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
    #[arg(
        short,
        long,
        help = "BWA executable to use",
        default_value = "bwa-mem2",
        value_parser = builder::PossibleValuesParser::new(["bwa", "bwa-mem2", "bwa-mem2.avx", "bwa-mem2.avx2", "bwa-mem2.avx512bw", "bwa-mem2.sse41", "bwa-mem2.sse42"])
    )]
    executable: String,
    #[arg(long, help = "Test mode: only list found samples without aligning")]
    dry_run: bool,
}

fn run_index(args: Index) {
    let mut bwa_index = BwaIndex::new(&args.reference);
    bwa_index.algorithm(args.algorithm);
    if let Some(prefix) = args.index_prefix {
        bwa_index.index_prefix(&prefix);
    }
    bwa_index.index();
}

fn run_align(args: Align) {
    let mut bwa_mem = BwaMem::new(&args.sample_name);
    bwa_mem
        .reference_path(&args.reference)
        .query_read1(&args.read1)
        .output_format(&args.output_format)
        .query_read2(args.read2)
        .output_path(&args.output);
    bwa_mem.align().expect("Failed to run BWA mem");
}

fn run_batch_align(args: BatchAlign) {
    let batch = BatchBwaAlign::new(&args.dir)
        .reference(&args.reference)
        .output(&args.output)
        .recursive(args.recursive)
        .threads(args.threads)
        .bwa_executable(&args.executable);
    if args.dry_run {
        batch.dry_run();
    } else {
        batch.run().expect("Failed to run batch BWA alignment");
    }
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

fn init_logger(file_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(dir) = file_path.parent() {
        fs::create_dir_all(dir)?;
    }
    let target = file_path.with_extension("log");
    let tofile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d(%Y-%m-%d %H:%M:%S %Z)} - {l} - {m}\n",
        )))
        .build(target)?;

    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{m}\n")))
        .build();

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(Appender::builder().build("logfile", Box::new(tofile)))
        .build(
            Root::builder()
                .appender("stdout")
                .appender("logfile")
                .build(LevelFilter::Info),
        )
        .expect("Failed building log configuration");

    log4rs::init_config(config).expect("Cannot initiate log configuration");

    Ok(())
}
