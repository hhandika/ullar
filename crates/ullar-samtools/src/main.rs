use std::path::PathBuf;

use clap::{Args, Parser, builder, crate_authors, crate_description, crate_name, crate_version};
use ullar_samtools::{
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
