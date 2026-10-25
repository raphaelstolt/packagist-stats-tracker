use clap::Parser;
use commands::Commands;

mod api;
mod commands;
mod db;
mod model;

#[derive(Parser)]
#[command(name = "packagist-stats-tracker", version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Track download statistics for Composer packages from Packagist")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    cli.command.run().await
}
