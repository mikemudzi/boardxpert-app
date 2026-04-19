use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "cut-optimizer-api")]
#[command(about = "2D cutting stock optimization API")]
pub struct Cli {
    /// Run as worker instead of API server
    #[arg(long)]
    pub worker: bool,

    /// Number of concurrent jobs (worker mode only)
    #[arg(long, default_value = "1")]
    pub concurrency: usize,
}
