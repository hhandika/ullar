use clap::Subcommand;

#[derive(Subcommand)]
pub(crate) enum DepsSubcommand {
    /// Check and manage dependencies
    #[command(name = "check", about = "Check and manage dependencies")]
    Check,
    /// Install dependencies
    #[command(name = "install", about = "Install dependencies")]
    Install,
    /// Update dependencies
    #[command(name = "update", about = "Update dependencies")]
    Update,
}
