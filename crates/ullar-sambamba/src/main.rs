use std::path::Path;

use clap::{Args, Parser, crate_authors, crate_description, crate_name, crate_version};
use ullar_sambamba::{
    batch::markdup::{BatchMarkDup, DEFAULT_MARKDUP_DIR},
    sambamba::markdup::SambambaMarkDup,
};

const LOG_FILE: &str = "ullar-sambamba.log";
fn main() {
    let cli = Cli::parse();
    ullar_logger::init_logger(Path::new(LOG_FILE)).expect("Failed to initialize logger");
    match cli {
        Cli::MarkDup(cmd) => run_markdup(cmd).expect("Failed to run markdup"),
        Cli::BatchMarkDup(cmd) => run_batch_markdup(cmd).expect("Failed to run batch markdup"),
    }
}

#[derive(Parser)]
#[command(name = crate_name!(), version = crate_version!(), about = crate_description!(), author = crate_authors!())]
enum Cli {
    #[command(
        name = "markdup",
        about = "Mark duplicates in a BAM file using Sambamba"
    )]
    MarkDup(MarkDupArgs),
    #[command(
        name = "batch-markdup",
        about = "Perform batch duplicate marking on a directory of BAM files"
    )]
    BatchMarkDup(BatchMarkDupArgs),
}

#[derive(Args)]
struct MarkDupArgs {
    #[arg(short, long, help = "Path to the input BAM file")]
    input: String,
    #[command(flatten)]
    common: CommonArgs,
    #[arg(short, long, help = "Path to the output BAM file or directory")]
    output: String,
}

#[derive(Args)]
struct BatchMarkDupArgs {
    #[arg(short, long, help = "Path to the directory containing BAM files")]
    dir: String,
    #[arg(short, long, help = "Process directories recursively")]
    recursive: bool,
    #[command(flatten)]
    common: CommonArgs,
    #[arg(short, long, default_value = DEFAULT_MARKDUP_DIR, help = "Path to the output BAM file or directory")]
    output: String,
    #[arg(short, long, help = "Perform a dry run without executing commands")]
    dry_run: bool,
}

#[derive(Args)]
struct CommonArgs {
    #[arg(
        short,
        long,
        help = "Sambamba executable to use",
        default_value = "sambamba"
    )]
    executable: Option<String>,
    #[arg(short, long, help = "Number of threads to use", default_value_t = 4)]
    threads: usize,
    #[arg(short, long, help = "Remove duplicates instead of marking them")]
    remove_duplicates: bool,
    #[arg(short, long, help = "Compression level (0-9) for output BAM files")]
    compression_level: Option<u8>,
    #[arg(long, help = "Override additional Sambamba options")]
    override_options: Option<String>,
}

fn run_markdup(args: MarkDupArgs) -> Result<(), Box<dyn std::error::Error>> {
    let mut sambamba = SambambaMarkDup::new(args.common.executable.as_deref());
    sambamba
        .input_bam(&args.input)
        .output_bam(&args.output)
        .threads(args.common.threads)
        .remove_duplicates(args.common.remove_duplicates)
        .compression_level(args.common.compression_level.unwrap_or(5));

    if let Some(ref options) = args.common.override_options {
        sambamba.override_options(options);
    }
    sambamba.execute()?;
    Ok(())
}

fn run_batch_markdup(args: BatchMarkDupArgs) -> Result<(), Box<dyn std::error::Error>> {
    let mut batch = BatchMarkDup::new(&args.dir);
    batch = batch
        .output_dir(&args.output)
        .recursive(args.recursive)
        .threads(args.common.threads)
        .remove_duplicates(args.common.remove_duplicates);

    if let Some(level) = args.common.compression_level {
        batch = batch.compression_level(level);
    }

    if let Some(ref options) = args.common.override_options {
        batch = batch.override_options(options);
    }

    if args.dry_run {
        batch.dry_run();
        return Ok(());
    }

    batch.execute()?;
    Ok(())
}
