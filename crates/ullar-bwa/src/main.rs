use clap::{Args, Parser, builder};
use ullar_bwa::bwa::subprocess::{BwaIndex, BwaMem};

fn main() {
    let cli = Cli::parse();

    match cli {
        Cli::Index(index_args) => run_index(index_args),
        Cli::Align(align_args) => run_align(align_args),
    }
}

#[derive(Parser)]
enum Cli {
    #[command(name = "index", about = "Index a reference genome using BWA")]
    Index(Index),
    #[command(name = "align", about = "Align reads to a reference genome using BWA")]
    Align(Align),
}

#[derive(Args)]
struct Index {
    #[arg(short, long, help = "Path to the reference file")]
    reference: String,
}

#[derive(Args)]
struct Align {
    #[arg(short, long, help = "Path to the reference file")]
    reference: String,
    #[arg(long, help = "Path to the query read 1 file")]
    read1: String,
    #[arg(long, help = "Path to the query read 2 file")]
    read2: Option<String>,
    #[arg(short = 'F', long, help = "Output format", default_value = "sam", value_parser = builder::PossibleValuesParser::new(["sam", "bam"]))]
    output_format: String,
    #[arg(short, long, help = "Path to the output file")]
    output: String,
}

fn run_index(args: Index) {
    let bwa = BwaIndex::build()
        .reference_path(std::path::Path::new(&args.reference))
        .index_prefix(std::path::Path::new("bwa_index"))
        .algorithm("is")
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
