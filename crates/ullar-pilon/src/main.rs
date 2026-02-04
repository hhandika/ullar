use clap::{Args, Parser, crate_authors, crate_description, crate_name, crate_version};
use ullar_pilon::batch::polish::BatchGenomePolishing;

fn main() {
    let cli = Cli::parse();
    ullar_logger::init_logger(std::path::Path::new("ullar-gatk.log"))
        .expect("Failed to initialize logger");
    match cli {
        Cli::Polish(polish_args) => {
            batch_polish_phased_bams(polish_args).expect("Failed to polish phased BAMs")
        }
    };
}

#[derive(Parser)]
#[command(name = crate_name!(), version = crate_version!(), about = crate_description!(), author = crate_authors!())]
enum Cli {
    #[command(name = "polish", about = "Sort a BAM file using GATK")]
    Polish(PolishPhasedBamsArgs),
}

#[derive(Args)]
struct PolishPhasedBamsArgs {
    #[arg(
        short,
        long,
        help = "Path to the input directory containing phased BAM files"
    )]
    dir: String,
    #[arg(long, help = "Recursively search for BAM files in subdirectories")]
    recursive: bool,
    #[arg(
        short,
        long,
        help = "Path to the reference directory containing genome FASTA files"
    )]
    reference_dir: String,
    #[arg(
        short,
        long,
        help = "Path to the output directory for polished genomes"
    )]
    output_path: String,
    #[arg(long, help = "Optional parameters to pass to Pilon")]
    optional_params: Vec<String>,
    #[arg(long, help = "Java options for Pilon (e.g., memory settings)")]
    java_options: Option<String>,
    #[arg(long, help = "Override default Pilon command options")]
    override_options: Option<String>,
    #[arg(long, help = "Path to the Pilon executable", default_value = "pilon")]
    exe: String,
}

fn batch_polish_phased_bams(args: PolishPhasedBamsArgs) -> Result<(), Box<dyn std::error::Error>> {
    let mut polisher = BatchGenomePolishing::new(Some(&args.exe));
    polisher
        .input_dir(&args.dir)
        .output_path(&args.output_path)
        .reference_dir(&args.reference_dir)
        .recursive(args.recursive)
        .optional_params(args.optional_params);
    if let Some(java_opts) = args.java_options {
        polisher.java_options(&java_opts);
    }
    if let Some(override_opts) = args.override_options {
        polisher.override_options(&override_opts);
    }
    polisher.polish_phased()
}
