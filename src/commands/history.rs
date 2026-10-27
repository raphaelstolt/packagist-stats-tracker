use anyhow::{Context, Result};
use clap::Args;
use comfy_table::{Row, Table};
use serde::Serialize;

use crate::commands::collect::{self, collect_package};
use crate::db;
use crate::model::Snapshot;

#[derive(Serialize)]
struct HistoryJsonOutput {
    package: String,
    start_date: String,
    end_date: String,
    snapshots: Vec<Snapshot>,
}

#[derive(Args)]
pub struct HistoryArgs {
    /// Package name (e.g. vendor/package)
    name: String,

    /// Time period to show (e.g. 7d, 30d, 90d)
    #[arg(short, long, default_value = "30d")]
    period: String,

    /// Output format (default: table)
    #[arg(long, value_enum, default_value_t = crate::commands::format::Format::Table)]
    format: crate::commands::format::Format,
}

/// Parse a period string like "7d", "30d", "90d" into the number of days.
fn parse_period_days(period: &str) -> Result<u32> {
    let period = period.trim().to_lowercase();
    if !period.ends_with('d') {
        anyhow::bail!(
            "Invalid period format: '{}'. Expected format like '7d', '30d', '90d'.",
            period
        );
    }
    let num_str = &period[..period.len() - 1];
    let days = num_str
        .parse::<u32>()
        .with_context(|| format!("Invalid period value: '{}'", num_str))?;
    if days == 0 {
        anyhow::bail!("Period must be greater than 0 days.");
    }
    Ok(days)
}

/// Calculate the start date by subtracting days from today.
fn get_start_date(days_ago: u32) -> Result<String> {
    // Get today's date from the system
    let output = std::process::Command::new("date")
        .arg("+%Y-%m-%d")
        .output()
        .context("Failed to run date command")?;

    if !output.status.success() {
        anyhow::bail!("date command failed");
    }

    let today_str = String::from_utf8(output.stdout)
        .context("Invalid date output")?
        .trim()
        .to_string();

    if today_str.is_empty() {
        anyhow::bail!("Empty date output");
    }

    // Parse today's date
    let parts: Vec<&str> = today_str.split('-').collect();
    if parts.len() != 3 {
        anyhow::bail!("Invalid date format: {}", today_str);
    }

    let year: i32 = parts[0].parse().context("Invalid year")?;
    let month: u32 = parts[1].parse().context("Invalid month")?;
    let day: u32 = parts[2].parse().context("Invalid day")?;

    // Calculate the start date by subtracting days using a simple approach
    let start_date =
        subtract_days(year, month, day, days_ago).context("Failed to calculate start date")?;

    Ok(start_date)
}

/// Subtract days from a date and return the result as YYYY-MM-DD.
fn subtract_days(year: i32, month: u32, day: u32, days_to_subtract: u32) -> Result<String> {
    let mut y = year;
    let mut m = month as i32;
    let mut d = day as i32 - days_to_subtract as i32;

    // Handle day underflow
    while d < 1 {
        m -= 1;
        if m < 1 {
            m = 12;
            y -= 1;
        }
        d += days_in_month(y, m as u32) as i32;
    }

    Ok(format!("{:04}-{:02}-{:02}", y, m, d))
}

/// Return the number of days in a given month/year.
fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

/// Check if a year is a leap year.
fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

impl HistoryArgs {
    pub async fn run(&self) -> Result<()> {
        let days = parse_period_days(&self.period)?;
        let start_date = get_start_date(days)?;
        let end_date = collect::get_today();

        let conn = db::connect()?;

        // Collect today's snapshot for this package first
        if self.format == crate::commands::format::Format::Table {
            print!("  Collecting latest data for {}... ", self.name);
        }
        match collect_package(&conn, &self.name).await {
            Ok(()) => {
                if self.format == crate::commands::format::Format::Table {
                    println!("✓");
                }
            }
            Err(e) => {
                if self.format == crate::commands::format::Format::Table {
                    eprintln!("✗ ({})", e);
                }
                // Continue showing history even if collect fails
            }
        }

        let snapshots =
            db::list_snapshots_by_name_and_date_range(&conn, &self.name, &start_date, &end_date)?;

        if self.format == crate::commands::format::Format::Json {
            let output = HistoryJsonOutput {
                package: self.name.clone(),
                start_date,
                end_date,
                snapshots,
            };
            let json = serde_json::to_string_pretty(&output)
                .context("Failed to serialize history output")?;
            println!("{}", json);
        } else {
            let mut table = Table::new();
            table.set_header(Row::from(["Date", "Version", "Downloads", "Daily"]));

            if snapshots.is_empty() {
                table.add_row(Row::from([format!(
                    "No snapshots found for '{}' between {} and {}",
                    self.name, start_date, end_date
                )]));
            } else {
                for snap in &snapshots {
                    table.add_row(Row::from([
                        snap.date.as_str(),
                        snap.latest_version.as_str(),
                        snap.downloads.to_string().as_str(),
                        snap.daily.to_string().as_str(),
                    ]));
                }
            }

            println!(
                "\nHistory for '{}' ({} to {}):\n{}",
                self.name, start_date, end_date, table
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_period_days_parses_valid_input() {
        assert_eq!(parse_period_days("7d").unwrap(), 7);
        assert_eq!(parse_period_days("30d").unwrap(), 30);
        assert_eq!(parse_period_days("90d").unwrap(), 90);
        assert_eq!(parse_period_days("1d").unwrap(), 1);
    }

    #[test]
    fn parse_period_days_rejects_invalid_input() {
        assert!(
            parse_period_days("7")
                .unwrap_err()
                .to_string()
                .contains("Invalid period format")
        );
        assert!(
            parse_period_days("abc")
                .unwrap_err()
                .to_string()
                .contains("Invalid period format")
        );
        assert!(
            parse_period_days("0d")
                .unwrap_err()
                .to_string()
                .contains("greater than 0")
        );
        assert!(
            parse_period_days("7w")
                .unwrap_err()
                .to_string()
                .contains("Invalid period")
        );
    }

    #[test]
    fn parse_period_days_is_case_insensitive() {
        assert_eq!(parse_period_days("30D").unwrap(), 30);
    }
}
