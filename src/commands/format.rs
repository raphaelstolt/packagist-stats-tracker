use clap::ValueEnum;

/// Output format for commands.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ValueEnum)]
pub enum Format {
    #[default]
    Table,
    Json,
}
