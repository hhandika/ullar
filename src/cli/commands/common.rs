use clap::Args;

#[derive(Args)]
pub struct CommonRunnerOptions {
    /// Should the SHA256 checksum be checked
    /// before assembling the files
    #[arg(long, help = "Process samples without checking SHA256 checksum")]
    pub ignore_checksum: bool,
    /// Process samples if true
    /// else check the config file only
    #[arg(
        long = "process",
        help = "Process samples if true else check for errors only"
    )]
    pub process: bool,
    /// Optional parameters for the assembly process
    #[arg(
        long,
        require_equals = true,
        help = "Optional parameters for the assembly process"
    )]
    pub override_args: Option<String>,
    /// Check config for errors
    #[arg(
        long,
        help = "Continue processing samples without checking the config file"
    )]
    pub skip_config_check: bool,
}
