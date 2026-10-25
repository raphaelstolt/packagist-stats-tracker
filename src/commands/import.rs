use anyhow::{Context, Result};
use clap::Args;
use std::io::Read;

use crate::db;
use crate::model::{Snapshot, TrackedPackage};

/// Import data structure matching the export JSON format.
#[derive(serde::Deserialize)]
struct ImportData {
    packages: Vec<TrackedPackage>,
    snapshots: Vec<Snapshot>,
}

#[derive(Args)]
pub struct ImportArgs {
    /// Input file path (defaults to stdin)
    #[arg(short, long)]
    input: Option<String>,
}

impl ImportArgs {
    pub fn run(&self) -> Result<()> {
        let json = match &self.input {
            Some(path) => std::fs::read_to_string(path)
                .with_context(|| format!("Failed to read import file: {}", path))?,
            None => {
                let mut buffer = String::new();
                std::io::stdin()
                    .read_to_string(&mut buffer)
                    .context("Failed to read from stdin")?;
                buffer
            }
        };

        let data: ImportData =
            serde_json::from_str(&json).context("Failed to parse import JSON")?;

        let conn = db::connect()?;
        let mut packages_imported = 0;
        let mut snapshots_imported = 0;

        for pkg in &data.packages {
            db::upsert_package(&conn, pkg)?;
            packages_imported += 1;
        }

        for snapshot in &data.snapshots {
            db::insert_snapshot(&conn, snapshot)?;
            snapshots_imported += 1;
        }

        println!(
            "Imported {} packages and {} snapshots",
            packages_imported, snapshots_imported
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn test_json() -> String {
        serde_json::json!({
            "packages": [
                {
                    "name": "monolog/monolog",
                    "latest_version": "3.8.1",
                    "downloads": 123456,
                    "daily": 500,
                    "last_updated": "2026-10-26T00:00:00+00:00"
                }
            ],
            "snapshots": [
                {
                    "date": "2026-10-26",
                    "name": "monolog/monolog",
                    "latest_version": "3.8.1",
                    "downloads": 123456,
                    "daily": 500
                }
            ]
        })
        .to_string()
    }

    #[test]
    fn import_data_parses_json() {
        let json = test_json();
        let data: ImportData = serde_json::from_str(&json).unwrap();
        assert_eq!(data.packages.len(), 1);
        assert_eq!(data.snapshots.len(), 1);
        assert_eq!(data.packages[0].name, "monolog/monolog");
    }

    #[test]
    fn import_upserts_packages_and_snapshots() {
        let conn = Connection::open_in_memory().unwrap();
        db::init_schema(&conn).unwrap();

        let json = test_json();
        let data: ImportData = serde_json::from_str(&json).unwrap();

        for pkg in &data.packages {
            db::upsert_package(&conn, pkg).unwrap();
        }
        for snapshot in &data.snapshots {
            db::insert_snapshot(&conn, snapshot).unwrap();
        }

        let packages = db::list_packages(&conn).unwrap();
        assert_eq!(packages.len(), 1);
        assert_eq!(packages[0].name, "monolog/monolog");

        let snapshots = db::list_all_snapshots(&conn).unwrap();
        assert_eq!(snapshots.len(), 1);
        assert_eq!(snapshots[0].date, "2026-10-26");
    }

    #[test]
    fn import_handles_empty_data() {
        let json = serde_json::json!({ "packages": [], "snapshots": [] }).to_string();
        let data: ImportData = serde_json::from_str(&json).unwrap();
        assert!(data.packages.is_empty());
        assert!(data.snapshots.is_empty());
    }

    #[test]
    fn import_args_struct_is_created() {
        let _args = ImportArgs { input: None };
    }
}
