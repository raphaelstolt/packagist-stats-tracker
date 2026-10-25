use anyhow::{Context, Result};
use directories::ProjectDirs;
use rusqlite::Connection;
use std::path::PathBuf;

use crate::model::{Snapshot, TrackedPackage};

/// Get the directory for storing application data.
pub fn data_dir() -> Result<PathBuf> {
    let proj_dirs = ProjectDirs::from("com", "packagist", "packagist-stats-tracker")
        .context("Could not determine data directory")?;
    Ok(proj_dirs.data_dir().to_path_buf())
}

/// Get the path to the SQLite database file.
pub fn db_path() -> Result<PathBuf> {
    Ok(data_dir()?.join("packages.db"))
}

/// Open a connection to the database, creating it and the schema if needed.
pub fn connect() -> Result<Connection> {
    let path = db_path()?;
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {:?}", parent))?;
    }

    let conn =
        Connection::open(&path).with_context(|| format!("Failed to open database: {:?}", path))?;
    init_schema(&conn)?;
    Ok(conn)
}

/// Create the packages and snapshots tables if they don't exist, and migrate existing tables.
pub fn init_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS packages (
            name TEXT PRIMARY KEY,
            latest_version TEXT NOT NULL DEFAULT '',
            downloads INTEGER NOT NULL DEFAULT 0,
            daily INTEGER NOT NULL DEFAULT 0,
            last_updated TEXT NOT NULL DEFAULT ''
        );
        CREATE TABLE IF NOT EXISTS snapshots (
            date TEXT NOT NULL,
            name TEXT NOT NULL,
            latest_version TEXT NOT NULL DEFAULT '',
            downloads INTEGER NOT NULL DEFAULT 0,
            daily INTEGER NOT NULL DEFAULT 0,
            PRIMARY KEY (date, name)
        );",
    )
    .context("Failed to create tables")?;

    // Migrate existing tables that may lack the `daily` column
    conn.execute(
        "ALTER TABLE packages ADD COLUMN daily INTEGER NOT NULL DEFAULT 0",
        [],
    )
    .ok(); // Ignore error if column already exists

    conn.execute(
        "ALTER TABLE snapshots ADD COLUMN daily INTEGER NOT NULL DEFAULT 0",
        [],
    )
    .ok(); // Ignore error if column already exists

    Ok(())
}

/// Insert or update a tracked package.
pub fn upsert_package(conn: &Connection, pkg: &TrackedPackage) -> Result<()> {
    conn.execute(
        "INSERT INTO packages (name, latest_version, downloads, daily, last_updated)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(name) DO UPDATE SET
             latest_version = excluded.latest_version,
             downloads = excluded.downloads,
             daily = excluded.daily,
             last_updated = excluded.last_updated",
        rusqlite::params![
            pkg.name,
            pkg.latest_version,
            pkg.downloads as i64,
            pkg.daily as i64,
            pkg.last_updated,
        ],
    )
    .context("Failed to upsert package")?;
    Ok(())
}

/// Check if a package is already tracked.
pub fn package_exists(conn: &Connection, name: &str) -> Result<bool> {
    let exists: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM packages WHERE name = ?1",
            rusqlite::params![name],
            |row| row.get(0),
        )
        .context("Failed to check package existence")?;
    Ok(exists)
}

/// List all tracked packages.
pub fn list_packages(conn: &Connection) -> Result<Vec<TrackedPackage>> {
    let mut stmt = conn
        .prepare("SELECT name, latest_version, downloads, daily, last_updated FROM packages ORDER BY name")
        .context("Failed to prepare list query")?;

    let packages = stmt
        .query_map(rusqlite::params![], |row| {
            Ok(TrackedPackage {
                name: row.get(0)?,
                latest_version: row.get(1)?,
                downloads: row.get::<_, i64>(2)? as u64,
                daily: row.get::<_, i64>(3)? as u64,
                last_updated: row.get(4)?,
            })
        })
        .context("Failed to query packages")?;

    packages.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

/// Insert or update a snapshot for a package on a given date.
pub fn insert_snapshot(conn: &Connection, snapshot: &Snapshot) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO snapshots (date, name, latest_version, downloads, daily)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![
            snapshot.date,
            snapshot.name,
            snapshot.latest_version,
            snapshot.downloads as i64,
            snapshot.daily as i64,
        ],
    )
    .context("Failed to insert snapshot")?;
    Ok(())
}

/// List snapshots for a specific package within a date range.
pub fn list_snapshots_by_name_and_date_range(
    conn: &Connection,
    name: &str,
    start_date: &str,
    end_date: &str,
) -> Result<Vec<Snapshot>> {
    let mut stmt = conn
        .prepare(
            "SELECT date, name, latest_version, downloads, daily \
             FROM snapshots \
             WHERE name = ?1 AND date >= ?2 AND date <= ?3 \
             ORDER BY date",
        )
        .context("Failed to prepare history query")?;

    let snapshots = stmt
        .query_map(rusqlite::params![name, start_date, end_date], |row| {
            Ok(Snapshot {
                date: row.get(0)?,
                name: row.get(1)?,
                latest_version: row.get(2)?,
                downloads: row.get::<_, i64>(3)? as u64,
                daily: row.get::<_, i64>(4)? as u64,
            })
        })
        .context("Failed to query snapshots")?;

    snapshots.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

/// List all snapshots for a given date.
pub fn list_snapshots_by_date(conn: &Connection, date: &str) -> Result<Vec<Snapshot>> {
    let mut stmt = conn
        .prepare("SELECT date, name, latest_version, downloads, daily FROM snapshots WHERE date = ?1 ORDER BY name")
        .context("Failed to prepare snapshot query")?;

    let snapshots = stmt
        .query_map(rusqlite::params![date], |row| {
            Ok(Snapshot {
                date: row.get(0)?,
                name: row.get(1)?,
                latest_version: row.get(2)?,
                downloads: row.get::<_, i64>(3)? as u64,
                daily: row.get::<_, i64>(4)? as u64,
            })
        })
        .context("Failed to query snapshots")?;

    snapshots.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

/// List all snapshots ordered by date descending, then by name.
pub fn list_all_snapshots(conn: &Connection) -> Result<Vec<Snapshot>> {
    let mut stmt = conn
        .prepare("SELECT date, name, latest_version, downloads, daily FROM snapshots ORDER BY date DESC, name")
        .context("Failed to prepare all snapshots query")?;

    let snapshots = stmt
        .query_map(rusqlite::params![], |row| {
            Ok(Snapshot {
                date: row.get(0)?,
                name: row.get(1)?,
                latest_version: row.get(2)?,
                downloads: row.get::<_, i64>(3)? as u64,
                daily: row.get::<_, i64>(4)? as u64,
            })
        })
        .context("Failed to query snapshots")?;

    snapshots.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn test_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init_schema(&conn).unwrap();
        conn
    }

    #[test]
    fn init_schema_creates_packages_table() {
        let conn = Connection::open_in_memory().unwrap();
        init_schema(&conn).unwrap();

        let table_exists: bool = conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='packages'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(table_exists);
    }

    #[test]
    fn upsert_package_inserts_new_package() {
        let conn = test_conn();
        let pkg = TrackedPackage {
            name: "monolog/monolog".to_string(),
            latest_version: "3.8.1".to_string(),
            downloads: 123456789,
            daily: 5000,
            last_updated: "2025-10-01T00:00:00+00:00".to_string(),
        };
        upsert_package(&conn, &pkg).unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM packages", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn upsert_package_updates_existing_package() {
        let conn = test_conn();
        let pkg1 = TrackedPackage {
            name: "monolog/monolog".to_string(),
            latest_version: "3.8.0".to_string(),
            downloads: 100,
            daily: 10,
            last_updated: "2025-09-01T00:00:00+00:00".to_string(),
        };
        upsert_package(&conn, &pkg1).unwrap();

        let pkg2 = TrackedPackage {
            name: "monolog/monolog".to_string(),
            latest_version: "3.8.1".to_string(),
            downloads: 200,
            daily: 20,
            last_updated: "2025-10-01T00:00:00+00:00".to_string(),
        };
        upsert_package(&conn, &pkg2).unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM packages", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);

        let version: String = conn
            .query_row(
                "SELECT latest_version FROM packages WHERE name = 'monolog/monolog'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(version, "3.8.1");
    }

    #[test]
    fn package_exists_returns_correct_value() {
        let conn = test_conn();
        assert!(!package_exists(&conn, "nonexistent/package").unwrap());

        let pkg = TrackedPackage {
            name: "monolog/monolog".to_string(),
            latest_version: "3.8.1".to_string(),
            downloads: 100,
            daily: 5,
            last_updated: "2025-10-01T00:00:00+00:00".to_string(),
        };
        upsert_package(&conn, &pkg).unwrap();

        assert!(package_exists(&conn, "monolog/monolog").unwrap());
    }

    #[test]
    fn list_packages_returns_all_packages_ordered() {
        let conn = test_conn();
        let packages = vec![
            TrackedPackage {
                name: "symfony/console".to_string(),
                latest_version: "7.2.0".to_string(),
                downloads: 500,
                daily: 25,
                last_updated: "2025-10-01T00:00:00+00:00".to_string(),
            },
            TrackedPackage {
                name: "monolog/monolog".to_string(),
                latest_version: "3.8.1".to_string(),
                downloads: 200,
                daily: 10,
                last_updated: "2025-10-01T00:00:00+00:00".to_string(),
            },
        ];
        for pkg in &packages {
            upsert_package(&conn, pkg).unwrap();
        }

        let listed = list_packages(&conn).unwrap();
        assert_eq!(listed.len(), 2);
        assert_eq!(listed[0].name, "monolog/monolog");
        assert_eq!(listed[1].name, "symfony/console");
    }

    #[test]
    fn init_schema_creates_snapshots_table() {
        let conn = Connection::open_in_memory().unwrap();
        init_schema(&conn).unwrap();

        let table_exists: bool = conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='snapshots'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(table_exists);
    }

    #[test]
    fn insert_snapshot_stores_snapshot() {
        let conn = test_conn();
        let snapshot = Snapshot {
            date: "2025-10-25".to_string(),
            name: "monolog/monolog".to_string(),
            latest_version: "3.8.1".to_string(),
            downloads: 123456,
            daily: 500,
        };
        insert_snapshot(&conn, &snapshot).unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM snapshots", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn list_snapshots_by_date_returns_snapshots_ordered() {
        let conn = test_conn();
        let snapshots = vec![
            Snapshot {
                date: "2025-10-25".to_string(),
                name: "symfony/console".to_string(),
                latest_version: "7.2.0".to_string(),
                downloads: 500,
                daily: 25,
            },
            Snapshot {
                date: "2025-10-25".to_string(),
                name: "monolog/monolog".to_string(),
                latest_version: "3.8.1".to_string(),
                downloads: 200,
                daily: 10,
            },
        ];
        for snap in &snapshots {
            insert_snapshot(&conn, snap).unwrap();
        }

        let listed = list_snapshots_by_date(&conn, "2025-10-25").unwrap();
        assert_eq!(listed.len(), 2);
        assert_eq!(listed[0].name, "monolog/monolog");
        assert_eq!(listed[1].name, "symfony/console");
    }

    #[test]
    fn list_snapshots_by_name_and_date_range_returns_ordered_by_date() {
        let conn = test_conn();
        let snapshots = vec![
            Snapshot {
                date: "2025-10-23".to_string(),
                name: "monolog/monolog".to_string(),
                latest_version: "3.8.0".to_string(),
                downloads: 100,
                daily: 5,
            },
            Snapshot {
                date: "2025-10-25".to_string(),
                name: "monolog/monolog".to_string(),
                latest_version: "3.8.1".to_string(),
                downloads: 200,
                daily: 10,
            },
            Snapshot {
                date: "2025-10-24".to_string(),
                name: "monolog/monolog".to_string(),
                latest_version: "3.8.1".to_string(),
                downloads: 150,
                daily: 8,
            },
            Snapshot {
                date: "2025-10-25".to_string(),
                name: "symfony/console".to_string(),
                latest_version: "7.2.0".to_string(),
                downloads: 500,
                daily: 25,
            },
        ];
        for snap in &snapshots {
            insert_snapshot(&conn, snap).unwrap();
        }

        let listed = list_snapshots_by_name_and_date_range(
            &conn,
            "monolog/monolog",
            "2025-10-23",
            "2025-10-25",
        )
        .unwrap();
        assert_eq!(listed.len(), 3);
        assert_eq!(listed[0].date, "2025-10-23");
        assert_eq!(listed[1].date, "2025-10-24");
        assert_eq!(listed[2].date, "2025-10-25");
    }

    #[test]
    fn list_snapshots_by_name_and_date_range_returns_empty_for_unknown_package() {
        let conn = test_conn();
        let listed =
            list_snapshots_by_name_and_date_range(&conn, "unknown/pkg", "2025-10-01", "2025-10-31")
                .unwrap();
        assert!(listed.is_empty());
    }
}
