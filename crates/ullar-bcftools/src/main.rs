use std::path::Path;

use clap::{Args, Parser, crate_authors, crate_description, crate_name, crate_version};
use ullar_bcftools::batch::BatchVariantCalling;

fn main() {
    let cli = Cli::parse();
    ullar_logger::init_logger(Path::new("ullar-bcftools.log"))
        .expect("Failed to initialize logger");
    match cli {
        Cli::Call(call_args) => run_variant_call(call_args).expect("Failed to run variant calling"),
    }
}

#[derive(Parser)]
#[command(name = crate_name!(), version = crate_version!(), about = crate_description!(), author = crate_authors!())]
enum Cli {
    #[command(
        name = "call",
        about = "Perform variant calling on BAM files in a directory using BCFtools"
    )]
    Call(VariantCall),
}

#[derive(Args)]
struct VariantCall {
    #[arg(short, long, help = "Path to the reference file")]
    reference: String,
    #[arg(short, long, help = "Path to the directory containing reads")]
    dir: String,
    #[arg(long, help = "Recursively search for reads in subdirectories")]
    recursive: bool,
    #[arg(short, long, help = "Path to the output directory for variant calls")]
    output_dir: String,
    #[arg(long, help = "Path to the BWA executable", default_value = "bwa-mem2")]
    exe: String,
    #[arg(
        long,
        help = "File name prefix for output VCF/BCF files",
        default_value = "variants"
    )]
    prefix: String,
    #[arg(long, help = "Ploidy level for variant calling (default: 2)")]
    ploidy: Option<u32>,
}

fn run_variant_call(args: VariantCall) -> Result<(), Box<dyn std::error::Error>> {
    let mut batch_caller = BatchVariantCalling::new(&args.dir);
    batch_caller
        .reference_path(&args.reference)
        .recursive(args.recursive)
        .output_dir(&args.output_dir)
        .prefix(&args.prefix);
    batch_caller.run()?;
    Ok(())
}
