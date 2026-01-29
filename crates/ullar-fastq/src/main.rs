use std::path::Path;

use clap::{Args, Parser};
use ullar_fastq::files::reader::FastqReader;

fn main() {
    let cli = Cli::parse();

    match cli {
        Cli::Description(desc_args) => run_description(desc_args),
    }
}

#[derive(Parser)]
enum Cli {
    #[command(name = "description", about = "Parse FASTQ header information")]
    Description(Description),
}

#[derive(Args)]
struct Description {
    #[arg(short, long, help = "Path to the FASTQ file")]
    input: String,
}

fn run_description(args: Description) {
    let path = Path::new(&args.input);
    let mut reader = match FastqReader::new(path) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error opening FASTQ file: {}", e);
            return;
        }
    };
    match reader.get_illumina_name() {
        Ok(h) => println!("Parsed Illumina Header: {}\n", h),
        Err(e) => {
            eprintln!("Error reading FASTQ header: {}", e);
            return;
        }
    }
}
