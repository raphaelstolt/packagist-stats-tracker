use anyhow::{Context, Result};
use clap::Args;
use comfy_table::{Row, Table};
use serde::Serialize;
use std::process;

use crate::api;
use crate::db;
use crate::model::Snapshot;

/// Get today's date as YYYY-MM-DD using the system date command.
pub fn get_today() -> String {
    let output = process::Command::new("date")
        .arg("+%Y-%m-%d")
        .output()
        .ok()
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .unwrap_or_else(|| "unknown".to_string());
    output.trim().to_string()
}

/// Collect a snapshot for a single package and store it in the database.
pub async fn collect_package(conn: &rusqlite::Connection, name: &str) -> Result<()> {
    let today = get_today();
    let tracked = api::fetch_package(name).await?;
    let snapshot = Snapshot {
        date: today,
        name: tracked.name.clone(),
        latest_version: tracked.latest_version.clone(),
        downloads: tracked.downloads,
        daily: tracked.daily,
    };
    db::insert_snapshot(conn, &snapshot)
        .with_context(|| format!("Failed to save snapshot for {}", name))?;
    Ok(())
}

#[derive(Serialize)]
struct CollectJsonOutput {
    date: String,
    collected: u64,
    failed: u64,
    snapshots: Vec<Snapshot>,
}

#[derive(Args)]
pub struct CollectArgs {
    /// Output format (default: table)
    #[arg(long, value_enum, default_value_t = crate::commands::format::Format::Table)]
    format: crate::commands::format::Format,
}

impl CollectArgs {
    pub async fn run(&self) -> Result<()> {
        let conn = db::connect()?;
        let packages = db::list_packages(&conn)?;

        if packages.is_empty() {
            println!("No packages tracked. Use 'package add' to add packages first.");
        } else {
            let today = get_today();

            if self.format == crate::commands::format::Format::Table {
                println!("Collecting snapshots for {}...", today);
            }
            let mut collected: u64 = 0;
            let mut failed: u64 = 0;

            for pkg in &packages {
                if self.format == crate::commands::format::Format::Table {
                    print!("  Fetching {}... ", pkg.name);
                }
                match collect_package(&conn, &pkg.name).await {
                    Ok(()) => {
                        if self.format == crate::commands::format::Format::Table {
                            println!("✓");
                        }
                        collected += 1;
                    }
                    Err(e) => {
                        if self.format == crate::commands::format::Format::Table {
                            eprintln!("✗ ({})", e);
                        }
                        failed += 1;
                    }
                }
            }

            let snapshots = db::list_snapshots_by_date(&conn, &today)?;

            if self.format == crate::commands::format::Format::Json {
                let output = CollectJsonOutput {
                    date: today,
                    collected,
                    failed,
                    snapshots,
                };
                let json = serde_json::to_string_pretty(&output)
                    .context("Failed to serialize collect output")?;
                println!("{}", json);
            } else {
                println!(
                    "\nCollection complete: {} succeeded, {} failed.",
                    collected, failed
                );

                if !snapshots.is_empty() {
                    let mut table = Table::new();
                    table.set_header(Row::from(["Package", "Version", "Downloads", "Daily"]));
                    for snap in &snapshots {
                        table.add_row(Row::from([
                            &snap.name,
                            &snap.latest_version,
                            &snap.downloads.to_string(),
                            &snap.daily.to_string(),
                        ]));
                    }
                    println!("\nToday's snapshots ({}):\n{}", today, table);
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collect_args_struct_is_created() {
        let _args = CollectArgs {
            format: crate::commands::format::Format::Table,
        };
    }

    #[test]
    fn snapshot_model_fields_are_correct() {
        let snapshot = Snapshot {
            date: "2025-10-25".to_string(),
            name: "monolog/monolog".to_string(),
            latest_version: "3.8.1".to_string(),
            downloads: 123456,
            daily: 500,
        };
        assert_eq!(snapshot.date, "2025-10-25");
        assert_eq!(snapshot.name, "monolog/monolog");
        assert_eq!(snapshot.latest_version, "3.8.1");
        assert_eq!(snapshot.downloads, 123456);
        assert_eq!(snapshot.daily, 500);
    }

    #[test]
    fn get_today_returns_valid_date_format() {
        let today = get_today();
        // Should match YYYY-MM-DD format
        assert!(
            today.len() == 10,
            "Date should be 10 characters, got: {}",
            today
        );
        assert!(
            today
                .chars()
                .enumerate()
                .all(|(i, c)| { (i == 4 || i == 7) && c == '-' || c.is_ascii_digit() }),
            "Date should be in YYYY-MM-DD format, got: {}",
            today
        );
    }
}
