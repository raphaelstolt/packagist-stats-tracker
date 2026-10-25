use anyhow::{Context, Result};
use clap::Args;
use comfy_table::{Row, Table};
use serde::Serialize;

use crate::commands::collect::{collect_package, get_today};
use crate::db;
use crate::model::Snapshot;

#[derive(Serialize)]
struct OverallJsonOutput {
    date: String,
    tracked_packages: u64,
    total_downloads: u64,
    daily_downloads: u64,
    snapshots: Vec<Snapshot>,
}

#[derive(Args)]
pub struct OverallArgs {
    /// Output format (default: table)
    #[arg(long, value_enum, default_value_t = crate::commands::format::Format::Table)]
    format: crate::commands::format::Format,
}

impl OverallArgs {
    pub async fn run(&self) -> Result<()> {
        let conn = db::connect()?;
        let today = get_today();

        // Collect fresh snapshots for all tracked packages
        let packages = db::list_packages(&conn)?;
        if !packages.is_empty() {
            for pkg in &packages {
                if let Err(e) = collect_package(&conn, &pkg.name).await
                    && self.format == crate::commands::format::Format::Table
                {
                    eprintln!("Warning: failed to collect '{}': {}", pkg.name, e);
                }
            }
        }

        let snapshots = db::list_snapshots_by_date(&conn, &today)?;

        if snapshots.is_empty() {
            if self.format == crate::commands::format::Format::Json {
                let output = OverallJsonOutput {
                    date: today.clone(),
                    tracked_packages: 0,
                    total_downloads: 0,
                    daily_downloads: 0,
                    snapshots: vec![],
                };
                let json = serde_json::to_string_pretty(&output)
                    .context("Failed to serialize overall output")?;
                println!("{}", json);
            } else {
                println!(
                    "No snapshots found for today ({}). Run 'collect' first.",
                    today
                );
            }
            return Ok(());
        }

        let total_packages = snapshots.len() as u64;
        let total_downloads: u64 = snapshots.iter().map(|s| s.downloads).sum();
        let total_daily: u64 = snapshots.iter().map(|s| s.daily).sum();

        if self.format == crate::commands::format::Format::Json {
            let mut sorted = snapshots.clone();
            sorted.sort_by_key(|s| std::cmp::Reverse(s.daily));
            let output = OverallJsonOutput {
                date: today,
                tracked_packages: total_packages,
                total_downloads,
                daily_downloads: total_daily,
                snapshots: sorted,
            };
            let json = serde_json::to_string_pretty(&output)
                .context("Failed to serialize overall output")?;
            println!("{}", json);
        } else {
            println!("Packagist Stats Overview ({})", today);
            println!("======================================");
            println!();
            println!("Tracked packages: {}", total_packages);
            println!("Total downloads:  {}", total_downloads);
            println!("Daily downloads:  {}", total_daily);

            // Sort by daily downloads descending for the top packages table
            let mut sorted = snapshots;
            sorted.sort_by_key(|s| std::cmp::Reverse(s.daily));

            println!();
            println!("Top packages by daily downloads:");
            let mut table = Table::new();
            table.set_header(Row::from(["Package", "Version", "Daily"]));
            for snap in &sorted {
                table.add_row(Row::from([
                    &snap.name,
                    &snap.latest_version,
                    &snap.daily.to_string(),
                ]));
            }
            println!("{}", table);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Snapshot;
    use rusqlite::Connection;

    fn test_conn_with_snapshots() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        db::init_schema(&conn).unwrap();
        let snapshots = vec![
            Snapshot {
                date: "2026-10-26".to_string(),
                name: "monolog/monolog".to_string(),
                latest_version: "3.8.1".to_string(),
                downloads: 100000,
                daily: 500,
            },
            Snapshot {
                date: "2026-10-26".to_string(),
                name: "symfony/console".to_string(),
                latest_version: "7.2.0".to_string(),
                downloads: 200000,
                daily: 1200,
            },
            Snapshot {
                date: "2026-10-26".to_string(),
                name: "guzzlehttp/guzzle".to_string(),
                latest_version: "7.9.2".to_string(),
                downloads: 150000,
                daily: 800,
            },
        ];
        for snap in &snapshots {
            db::insert_snapshot(&conn, snap).unwrap();
        }
        conn
    }

    #[test]
    fn overall_args_struct_is_created() {
        let _args = OverallArgs {
            format: crate::commands::format::Format::Table,
        };
    }

    #[test]
    fn overall_aggregates_totals_correctly() {
        let conn = test_conn_with_snapshots();
        let snapshots = db::list_snapshots_by_date(&conn, "2026-10-26").unwrap();

        let total_packages = snapshots.len();
        let total_downloads: u64 = snapshots.iter().map(|s| s.downloads).sum();
        let total_daily: u64 = snapshots.iter().map(|s| s.daily).sum();

        assert_eq!(total_packages, 3);
        assert_eq!(total_downloads, 450000);
        assert_eq!(total_daily, 2500);
    }

    #[test]
    fn overall_sorts_by_daily_descending() {
        let conn = test_conn_with_snapshots();
        let mut snapshots = db::list_snapshots_by_date(&conn, "2026-10-26").unwrap();
        snapshots.sort_by_key(|s| std::cmp::Reverse(s.daily));

        assert_eq!(snapshots[0].name, "symfony/console");
        assert_eq!(snapshots[0].daily, 1200);
        assert_eq!(snapshots[1].name, "guzzlehttp/guzzle");
        assert_eq!(snapshots[1].daily, 800);
        assert_eq!(snapshots[2].name, "monolog/monolog");
        assert_eq!(snapshots[2].daily, 500);
    }
}
