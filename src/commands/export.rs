use anyhow::{Context, Result};
use clap::Args;

use crate::db;

/// Export data structure for JSON output.
#[derive(serde::Serialize)]
struct ExportData {
    packages: Vec<crate::model::TrackedPackage>,
    snapshots: Vec<crate::model::Snapshot>,
}

#[derive(Args)]
pub struct ExportArgs {
    /// Output file path (defaults to stdout)
    #[arg(short, long)]
    output: Option<String>,
}

impl ExportArgs {
    pub fn run(&self) -> Result<()> {
        let conn = db::connect()?;
        let packages = db::list_packages(&conn)?;
        let snapshots = db::list_all_snapshots(&conn)?;

        let data = ExportData {
            packages,
            snapshots,
        };

        let json =
            serde_json::to_string_pretty(&data).context("Failed to serialize data to JSON")?;

        match &self.output {
            Some(path) => {
                std::fs::write(path, &json)
                    .with_context(|| format!("Failed to write export file: {}", path))?;
                println!(
                    "Exported {} packages and {} snapshots to {}",
                    data.packages.len(),
                    data.snapshots.len(),
                    path
                );
            }
            None => {
                println!("{}", json);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Snapshot, TrackedPackage};
    use rusqlite::Connection;

    fn test_conn_with_data() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        db::init_schema(&conn).unwrap();

        let pkg = TrackedPackage {
            name: "monolog/monolog".to_string(),
            latest_version: "3.8.1".to_string(),
            downloads: 123456,
            daily: 500,
            last_updated: "2026-10-26T00:00:00+00:00".to_string(),
        };
        db::upsert_package(&conn, &pkg).unwrap();

        let snapshot = Snapshot {
            date: "2026-10-26".to_string(),
            name: "monolog/monolog".to_string(),
            latest_version: "3.8.1".to_string(),
            downloads: 123456,
            daily: 500,
        };
        db::insert_snapshot(&conn, &snapshot).unwrap();

        conn
    }

    #[test]
    fn export_args_struct_is_created() {
        let _args = ExportArgs { output: None };
    }

    #[test]
    fn export_data_serializes_to_json() {
        let conn = test_conn_with_data();
        let packages = db::list_packages(&conn).unwrap();
        let snapshots = db::list_all_snapshots(&conn).unwrap();

        let data = ExportData {
            packages,
            snapshots,
        };

        let json = serde_json::to_string_pretty(&data).unwrap();
        assert!(json.contains("monolog/monolog"));
        assert!(json.contains("packages"));
        assert!(json.contains("snapshots"));
    }

    #[test]
    fn export_data_includes_all_packages() {
        let conn = test_conn_with_data();
        let packages = db::list_packages(&conn).unwrap();
        assert_eq!(packages.len(), 1);
        assert_eq!(packages[0].name, "monolog/monolog");
    }

    #[test]
    fn export_data_includes_all_snapshots() {
        let conn = test_conn_with_data();
        let snapshots = db::list_all_snapshots(&conn).unwrap();
        assert_eq!(snapshots.len(), 1);
        assert_eq!(snapshots[0].name, "monolog/monolog");
        assert_eq!(snapshots[0].date, "2026-10-26");
    }
}
