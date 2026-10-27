pub mod collect;
pub mod export;
pub mod format;
pub mod history;
pub mod import;
pub mod overall;
pub mod package;

use anyhow::Result;
use clap::{Args, Subcommand};

use self::collect::CollectArgs;
use self::export::ExportArgs;
use self::history::HistoryArgs;
use self::import::ImportArgs;
use self::overall::OverallArgs;
use self::package::{AddArgs, ListArgs as PackageListArgs, RemoveArgs};

#[derive(Subcommand)]
pub enum PackageCommands {
    /// Add a package to track
    Add(AddArgs),
    /// List all tracked packages
    List(PackageListArgs),
    /// Remove a package from tracking
    Remove(RemoveArgs),
}

impl PackageCommands {
    pub async fn run(&self) -> Result<()> {
        match self {
            PackageCommands::Add(args) => args.run().await,
            PackageCommands::List(args) => args.run(),
            PackageCommands::Remove(args) => args.run(),
        }
    }
}

#[derive(Args)]
pub struct PackageArgs {
    #[command(subcommand)]
    command: Option<PackageCommands>,
}

impl PackageArgs {
    pub async fn run(&self) -> Result<()> {
        match &self.command {
            Some(cmd) => cmd.run().await,
            None => {
                println!("Usage: packagist-stats-tracker package <COMMAND>\n\nCommands:");
                println!("  add    Add a package to track");
                println!("  list   List all tracked packages");
                println!("  remove Remove a package from tracking");
                Ok(())
            }
        }
    }
}

#[derive(Subcommand)]
pub enum Commands {
    /// Collect today's snapshots for all tracked packages
    Collect(CollectArgs),
    /// Export tracked packages and snapshots as JSON
    Export(ExportArgs),
    /// Import packages and snapshots from a JSON file
    Import(ImportArgs),
    /// Show download history for a package
    History(HistoryArgs),
    /// Show an overview of all tracked packages
    Overall(OverallArgs),
    /// Package management commands
    Package(PackageArgs),
}

impl Commands {
    pub async fn run(&self) -> Result<()> {
        match self {
            Commands::Collect(args) => args.run().await,
            Commands::Export(args) => args.run(),
            Commands::Import(args) => args.run(),
            Commands::History(args) => args.run().await,
            Commands::Overall(args) => args.run().await,
            Commands::Package(args) => args.run().await,
        }
    }
}
