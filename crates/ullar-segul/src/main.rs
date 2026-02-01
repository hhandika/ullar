use clap::{Args, Parser, builder, crate_authors, crate_description, crate_name, crate_version};
use std::path::Path;
use ullar_segul::sequence::transpose::TransposeSequence;

fn main() {
    ullar_logger::init_logger(Path::new("ullar_segul.log")).expect("Failed to initialize logger");
    let cli = Cli::parse();
    match cli {
        Cli::Transpose(transpose_args) => {
            run_transpose(transpose_args).expect("Failed to run transpose")
        }
    }
}

#[derive(Parser)]
#[command(name = crate_name!(), version = crate_version!(), about = crate_description!(), author = crate_authors!())]
enum Cli {
    #[command(
        name = "transpose",
        about = "Transpose sequences from locus alignments to create individual-level reference sequences"
    )]
    Transpose(TransposeArgs),
}

#[derive(Args)]
struct TransposeArgs {
    #[arg(short, long, help = "Input sequence files", required = true)]
    dir: String,
    #[arg(
        short = 'f',
        long,
        help = "Input format (fasta, nexus, phylip)",
        default_value = "fasta",
        value_parser = builder::PossibleValuesParser::new(["fasta", "nexus", "phylip"])
    )]
    input_fmt: String,
    #[arg(
        short,
        long,
        help = "Output directory",
        default_value = "./transposed_sequences"
    )]
    output: String,
}

fn run_transpose(args: TransposeArgs) -> Result<(), Box<dyn std::error::Error>> {
    let mut transposer = TransposeSequence::new(&args.dir, &args.input_fmt);
    transposer.output_dir(&args.output);
    transposer.transpose()?;
    Ok(())
}
