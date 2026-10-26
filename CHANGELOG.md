# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed
- The logo is removed.

## [0.1.2] - 2026-10-26

### Fixed
- The logo is really shipped.

## [0.1.1] - 2026-10-26

### Fixed
- The logo is also shipped.

## [0.1.0] - 2026-10-26

### Added
- `package add` command to track Composer packages from Packagist with automatic initial snapshot collection
- `package list` command to display all tracked packages with version, downloads, and daily stats
- `collect` command to collect daily snapshots for all tracked packages (updates existing data on re-run)
- `history` command to view download history for a package over a configurable period (format: `Nd`, default `30d`)
- `overall` command to display aggregated statistics across all tracked packages
- `export` command to backup all packages and snapshots as JSON
- `import` command to restore packages and snapshots from exported JSON
- `--format json` option on `collect`, `history`, and `overall` commands for machine-readable output
- Local SQLite storage for daily snapshots with no database server required
- Packagist API integration for fetching package details and download statistics
