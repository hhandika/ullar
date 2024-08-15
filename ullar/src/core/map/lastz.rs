//! Map contigs to reference using LASTZ.
pub const DEFAULT_LASTZ_PARAMS: &str = "datasets/contigs/Bunomys_chrysocomus_LSUMZ39568/contigs.fasta[multiple,nameparse=full] datasets/uce-5k-probes.fasta --nogfextend --step=20 --gap=400,30 --format=maf --strand=both > results.maf";

pub const DEFAULT_LASTZ_PARAMS_2: &str =
    "--step=20 --gap=400,30 --format=maf --strand=both > results.maf";
