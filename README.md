# packagist-stats-tracker

A CLI tool for tracking Packagist package download statistics over time.

`packagist-stats-tracker` collects download statistics for the PHP packages you care about, stores daily snapshots locally
in SQLite, and lets you inspect historical trends and compare packages from your terminal.

## Why?

Packagist provides package download statistics, but checking multiple packages repeatedly makes it difficult to build a
consistent historical picture.

`packagist-stats-tracker` automates collection and maintains a local history, so you can monitor package activity over
time instead of repeatedly checking individual Packagist pages.

## Features

- Add packages to your tracking list.
- Remove packages from your tracking list.
- List tracked packages.
- Collect statistics for all tracked packages.
- Store daily snapshots locally.
- Review a package's download history over a configurable period.
- View an overview of all tracked packages with aggregated statistics.
- Export all data as JSON for backup or migration.
- Import previously exported data to restore packages and snapshots.
- Output command results as JSON for scripting and automation.
- Use familiar terminal commands with built-in help.

## Installation

### From crates.io

```bash
cargo install packagist-stats-tracker
```

### From source

```bash
git clone git@github.com:raphaelstolt/packagist-stats-tracker.git
cd packagist-stats-tracker
cargo install --path .
```

This places the `packagist-stats-tracker` binary in `~/.cargo/bin`, which should be on your PATH.

## Usage

### Add a package

```bash
packagist-stats-tracker package add monolog/monolog
```

Add multiple packages at once:

```bash
packagist-stats-tracker package add monolog/monolog symfony/console guzzlehttp/guzzle
```

Adds the package(s) to your tracking list and immediately collects an initial snapshot for each, so you don't need to
run `collect` separately after adding new packages.

### List tracked packages

```bash
packagist-stats-tracker package list
```

### Remove a package

```bash
packagist-stats-tracker package remove monolog/monolog
```

Remove multiple packages at once:

```bash
packagist-stats-tracker package remove monolog/monolog symfony/console
```

Removes the package(s) from your tracking list and deletes all associated historical snapshots. If a package is not
tracked, the command will fail with an error.

### Collect statistics

Collect snapshots for every package in your tracking list:

```bash
packagist-stats-tracker collect
```

Run this command regularly to build a useful history. Running it again on the same day updates that day's data rather
than creating another daily entry.

### View download history

```bash
packagist-stats-tracker history monolog/monolog
```

Before showing the history, the command automatically collects the latest snapshot for the requested package, so you
always see today's data even if you haven't run `collect` yet.

Choose a period with `--period`:

```bash
packagist-stats-tracker history monolog/monolog --period 7d
packagist-stats-tracker history monolog/monolog --period 30d
packagist-stats-tracker history monolog/monolog --period 90d
```

The period uses the format `Nd`, where `N` is the number of days. The default period is `30d`.

### View overall statistics

Display a dashboard of all tracked packages with aggregated download totals and a ranking by daily downloads:

```bash
packagist-stats-tracker overall
```

Shows the number of tracked packages, combined total downloads, combined daily downloads, and a table of packages
sorted by daily downloads in descending order. Before displaying, it automatically collects the latest snapshot for all
tracked packages.

Example output:

```
Packagist Stats Overview (2026-10-26)
======================================

Tracked packages: 3
Total downloads:  450000
Daily downloads:  2500

Top packages by daily downloads:
┌───────────────────────┬───────────┬───────┐
│ Package               │ Version   │ Daily │
├───────────────────────┼───────────┼───────┤
│ symfony/console       │ 7.2.0     │ 1200  │
│ guzzlehttp/guzzle     │ 7.9.2     │ 800   │
│ monolog/monolog       │ 3.8.1     │ 500   │
└───────────────────────┴───────────┴───────┘
```

### Export data

Export all tracked packages and snapshots as JSON, useful for backup before upgrading:

```bash
packagist-stats-tracker export
```

Save to a file:

```bash
packagist-stats-tracker export -o backup.json
```

### Import data

Restore packages and snapshots from a previously exported JSON file:

```bash
packagist-stats-tracker import -i backup.json
```

Or pipe the data via stdin:

```bash
cat backup.json | packagist-stats-tracker import
```

### JSON output format

The `collect`, `history`, and `overall` commands support `--format json` for machine-readable output:

```bash
packagist-stats-tracker collect --format json
packagist-stats-tracker history monolog/monolog --format json
packagist-stats-tracker overall --format json
```

Example output for the `overall` command:

```json
{
  "date": "2026-10-26",
  "tracked_packages": 3,
  "total_downloads": 450000,
  "daily_downloads": 2500,
  "snapshots": [
    {
      "date": "2026-10-26",
      "name": "symfony/console",
      "latest_version": "7.2.0",
      "downloads": 200000,
      "daily": 1200
    },
    {
      "date": "2026-10-26",
      "name": "guzzlehttp/guzzle",
      "latest_version": "7.9.2",
      "downloads": 150000,
      "daily": 800
    },
    {
      "date": "2026-10-26",
      "name": "monolog/monolog",
      "latest_version": "3.8.1",
      "downloads": 100000,
      "daily": 500
    }
  ]
}
```

### Automate collection with cron

To collect statistics daily, add a cron job:

```bash
# Collect at 8:00 AM every day
0 8 * * * /home/user/.cargo/bin/packagist-stats-tracker collect >> /var/log/packagist-stats.log 2>&1
```

### Get help

```bash
packagist-stats-tracker --help
packagist-stats-tracker package --help
```

## Data storage

The application uses SQLite to store local snapshots and the `directories` crate to resolve platform-specific directories.
No separate database server is required.

Run `collect` regularly—such as from cron or another scheduler—to keep the historical data up to date. The history
available to the tool depends on when you started collecting snapshots.

## AI disclaimer

This package is developed with the assistance of Artificial Intelligence (AI). AI tools are used throughout the development
process, including code generation, documentation, testing, and maintenance.

While reasonable care is taken to review and validate AI-generated contributions, AI-generated code is not inherently correct,
secure, or free of defects. All contributions remain subject to human review and project quality standards.

As with any software, use this package at your own risk. No guarantee is made regarding its fitness for a particular purpose,
and the maintainers assume no liability for damages resulting from its use.

## Development

Build the project in debug mode:

```bash
cargo build
```

Build an optimized binary:

```bash
cargo build --release
```

For code quality guidelines (formatting, linting, and testing), please see [CONTRIBUTING.md](CONTRIBUTING.md).

## License

This project is licensed under the MIT license. Please see [LICENSE.md](LICENSE.md) for more details.
