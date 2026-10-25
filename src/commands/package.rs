use anyhow::{Context, Result, bail};
use clap::Args;
use comfy_table::{Row, Table};

use crate::api;
use crate::commands::collect::collect_package;
use crate::db;

#[derive(Args)]
pub struct AddArgs {
    /// Package name in vendor/name format (e.g., monolog/monolog)
    pub package: String,
}

impl AddArgs {
    pub async fn run(&self) -> Result<()> {
        let name = self.package.trim();

        // Validate package name format
        if !name.contains('/') {
            bail!(
                "Invalid package name '{}'. Expected format: vendor/package (e.g., monolog/monolog)",
                name
            );
        }

        let conn = db::connect()?;

        // Check if already tracked
        if db::package_exists(&conn, name)? {
            println!("Package '{}' is already tracked. Updating...", name);
        } else {
            println!("Fetching package '{}' from Packagist...", name);
        }

        // Fetch from Packagist API
        let tracked = api::fetch_package(name)
            .await
            .with_context(|| format!("Failed to fetch package '{}'", name))?;

        // Store in database
        db::upsert_package(&conn, &tracked)
            .with_context(|| format!("Failed to save package '{}'", name))?;

        // Collect initial snapshot
        collect_package(&conn, name)
            .await
            .with_context(|| format!("Failed to collect initial snapshot for '{}'", name))?;

        println!(
            "✓ '{}' tracked (version: {}, downloads: {}, daily: {})",
            tracked.name, tracked.latest_version, tracked.downloads, tracked.daily
        );
        Ok(())
    }
}

#[derive(Args)]
pub struct ListArgs;

impl ListArgs {
    pub fn run(&self) -> Result<()> {
        let conn = db::connect()?;
        let packages = db::list_packages(&conn)?;

        let mut table = Table::new();
        table.set_header(Row::from([
            "Package",
            "Version",
            "Downloads",
            "Daily",
            "Last Updated",
        ]));

        if packages.is_empty() {
            table.add_row(Row::from(["No packages tracked yet"]));
        } else {
            for pkg in &packages {
                table.add_row(Row::from([
                    &pkg.name,
                    &pkg.latest_version,
                    &pkg.downloads.to_string(),
                    &pkg.daily.to_string(),
                    &pkg.last_updated,
                ]));
            }
        }

        println!("{table}");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use comfy_table::{Row, Table};

    #[test]
    fn add_args_validates_package_name_format() {
        let args = AddArgs {
            package: "invalidname".to_string(),
        };
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let result = rt.block_on(args.run());
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid package name")
        );
    }

    #[test]
    fn list_command_shows_empty_message_when_no_packages() {
        let mut table = Table::new();
        table.set_header(Row::from([
            "Package",
            "Version",
            "Downloads",
            "Daily",
            "Last Updated",
        ]));
        table.add_row(Row::from(["No packages tracked yet"]));

        let output = format!("{table}");
        assert!(output.contains("Package"));
        assert!(output.contains("No packages tracked yet"));
    }

    #[test]
    fn list_command_shows_packages_in_table() {
        let mut table = Table::new();
        table.set_header(Row::from([
            "Package",
            "Version",
            "Downloads",
            "Daily",
            "Last Updated",
        ]));
        table.add_row(Row::from(&[
            "monolog/monolog",
            "3.8.1",
            "123456",
            "500",
            "2025-10-01",
        ]));

        let output = format!("{table}");
        assert!(output.contains("monolog/monolog"));
        assert!(output.contains("3.8.1"));
        assert!(output.contains("123456"));
        assert!(output.contains("500"));
    }
}
