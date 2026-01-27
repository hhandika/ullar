use clap::{Args, Parser, builder};
use ullar_gatk::gatk::sort::GatkSort;

fn main() {
    let cli = Cli::parse();

    match cli {
        Cli::Sort(sort_args) => run_sort(sort_args),
    }
}

#[derive(Parser)]
enum Cli {
    #[command(name = "sort", about = "Sort a BAM file using GATK")]
    Sort(Sort),
}

#[derive(Args)]
struct Sort {
    #[arg(short, long, help = "Path to the input BAM/CRAM file")]
    input: String,
    #[arg(
        short,
        long,
        default_value = "sorted_bam",
        help = "Path to the output BAM/CRAM file"
    )]
    output: String,
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

fn run_sort(args: Sort) {
    let executable = args.exe.to_owned();
    let gatk_sort = GatkSort::new(executable)
        .input_path(args.input)
        .output_path(args.output)
        .sort_order(&args.sort_order)
        .create_index(args.create_index);

    if let Some(temp_dir) = args.temp_dir {
        gatk_sort.temp_dir(temp_dir);
    }
    gatk_sort.execute();
}
