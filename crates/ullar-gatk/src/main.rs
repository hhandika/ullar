use clap::{Args, Parser, builder, crate_authors, crate_description, crate_name, crate_version};
use ullar_gatk::{
    batch::variant_calling::{BatchVariantCalling, DEFAULT_VARIANT_CALLING_DIR},
    gatk::sort::GatkSort,
};
fn main() {
    let cli = Cli::parse();
    ullar_logger::init_logger(std::path::Path::new("ullar-gatk.log"))
        .expect("Failed to initialize logger");
    match cli {
        Cli::Sort(sort_args) => run_sort(sort_args),
        Cli::HaplotypeCaller(hc_args) => batch_haplotype_calling(hc_args),
        _ => unimplemented!("This command is not yet implemented."),
    }
}

#[derive(Parser)]
#[command(name = crate_name!(), version = crate_version!(), about = crate_description!(), author = crate_authors!())]
enum Cli {
    #[command(name = "sort", about = "Sort a BAM file using GATK")]
    Sort(Sort),
    #[command(
        name = "variant",
        about = "Batch call variants using GATK HaplotypeCaller"
    )]
    HaplotypeCaller(HaplotypeCallerArgs),
    #[command(
        name = "group",
        about = "Add or replace read groups in a BAM file using GATK"
    )]
    AddReplaceGroups(AddReplaceGroups),
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

#[derive(Args)]
struct HaplotypeCallerArgs {
    #[arg(short, long, help = "Path to the input BAM/CRAM file")]
    dir: String,
    #[arg(long, help = "Process directories recursively")]
    recursive: bool,
    #[arg(short, long, help = "Path to the reference genome FASTA file")]
    reference: String,
    #[arg(
        short,
        long,
        default_value = DEFAULT_VARIANT_CALLING_DIR,
        help = "Path to the output VCF file"
    )]
    output: String,
    #[arg(long, help = "Ploidy of the sample", default_value_t = 2)]
    ploidy: u8,
    #[arg(long, help = "Java options for GATK")]
    java_options: Option<String>,
    #[arg(long, help = "Additional optional parameters for GATK")]
    optional_params: Vec<String>,
    #[arg(long, help = "Override options for GATK")]
    override_options: Option<String>,
    #[arg(long, help = "Path to the GATK executable")]
    exe: Option<String>,
}

#[derive(Args)]
struct AddReplaceGroups {
    #[arg(short, long, help = "Path to the input BAM/CRAM file")]
    input: String,
    #[arg(
        short,
        long,
        default_value = "rg_added_bam",
        help = "Path to the output BAM/CRAM file"
    )]
    output: String,
    #[arg(long, help = "Read group ID")]
    rg_id: String,
    #[arg(long, help = "Read group sample name")]
    rg_sm: String,
    #[arg(long, help = "Read group library")]
    rg_lb: Option<String>,
    #[arg(long, help = "Read group platform")]
    rg_pl: Option<String>,
    #[arg(long, help = "Path to the GATK executable")]
    exe: Option<String>,
}

fn run_sort(args: Sort) {
    let mut gatk_sort = GatkSort::new(args.exe.as_deref());
    gatk_sort
        .input_path(args.input)
        .output_path(args.output)
        .sort_order(&args.sort_order)
        .create_index(args.create_index);

    if let Some(temp_dir) = args.temp_dir {
        gatk_sort.temp_dir(temp_dir.to_string());
    }
    gatk_sort.execute();
}

fn batch_haplotype_calling(args: HaplotypeCallerArgs) {
    let mut gatk_hc = BatchVariantCalling::new(args.dir);
    gatk_hc
        .reference(args.reference)
        .output_dir(args.output)
        .recursive(args.recursive)
        .ploidy(args.ploidy);

    if let Some(exe) = args.exe {
        gatk_hc.exe(&exe);
    }

    if let Some(java_opts) = args.java_options {
        gatk_hc.java_options(&java_opts);
    }

    if !args.optional_params.is_empty() {
        gatk_hc.optional_params(args.optional_params);
    }

    if let Some(override_opts) = args.override_options {
        gatk_hc.override_options(&override_opts);
    }

    gatk_hc.execute().expect("Batch variant calling failed");
}
