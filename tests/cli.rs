use assert_cmd::Command;

/// Basic smoke test: the CLI should respond to --help
#[test]
fn cli_shows_help() {
    let mut cmd = Command::cargo_bin("packagist-stats-tracker").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("packagist-stats-tracker").count(1))
        .stdout(predicates::str::contains(
            "Track download statistics for Composer packages from Packagist",
        ));
}

/// The `package` subcommand should show help with add and list
#[test]
fn package_subcommand_shows_help() {
    let mut cmd = Command::cargo_bin("packagist-stats-tracker").unwrap();
    cmd.args(["package", "--help"])
        .assert()
        .success()
        .stdout(predicates::str::contains("add"))
        .stdout(predicates::str::contains("list"));
}

/// The `package add` subcommand should require a package argument
#[test]
fn package_add_requires_argument() {
    let mut cmd = Command::cargo_bin("packagist-stats-tracker").unwrap();
    cmd.arg("package")
        .arg("add")
        .assert()
        .failure()
        .stderr(predicates::str::contains("required"));
}

/// The `package add` subcommand should reject invalid package names
#[test]
fn package_add_rejects_invalid_name() {
    let mut cmd = Command::cargo_bin("packagist-stats-tracker").unwrap();
    cmd.args(["package", "add", "invalidname"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("Invalid package name"));
}

/// The `package list` subcommand should run successfully
#[test]
fn package_list_runs() {
    let mut cmd = Command::cargo_bin("packagist-stats-tracker").unwrap();
    cmd.args(["package", "list"])
        .assert()
        .success()
        .stdout(predicates::str::contains("Package"));
}

/// The `package list` subcommand should show table headers
#[test]
fn package_list_shows_headers() {
    let mut cmd = Command::cargo_bin("packagist-stats-tracker").unwrap();
    cmd.args(["package", "list"])
        .assert()
        .success()
        .stdout(predicates::str::contains("Package"))
        .stdout(predicates::str::contains("Version"))
        .stdout(predicates::str::contains("Downloads"))
        .stdout(predicates::str::contains("Last Updated"));
}

/// The `collect` subcommand should show help
#[test]
fn collect_subcommand_shows_help() {
    let mut cmd = Command::cargo_bin("packagist-stats-tracker").unwrap();
    cmd.args(["collect", "--help"])
        .assert()
        .success()
        .stdout(predicates::str::contains("Collect today's snapshots"));
}

/// The `collect` subcommand should run successfully (even with no packages)
#[test]
fn collect_command_runs() {
    let mut cmd = Command::cargo_bin("packagist-stats-tracker").unwrap();
    cmd.arg("collect").assert().success();
}

/// The main help should list the collect command
#[test]
fn main_help_shows_collect() {
    let mut cmd = Command::cargo_bin("packagist-stats-tracker").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("collect"));
}

/// The `history` subcommand should show help
#[test]
fn history_subcommand_shows_help() {
    let mut cmd = Command::cargo_bin("packagist-stats-tracker").unwrap();
    cmd.args(["history", "--help"])
        .assert()
        .success()
        .stdout(predicates::str::contains("Show download history"));
}

/// The `history` subcommand should require a package name argument
#[test]
fn history_requires_package_name() {
    let mut cmd = Command::cargo_bin("packagist-stats-tracker").unwrap();
    cmd.arg("history")
        .assert()
        .failure()
        .stderr(predicates::str::contains("required"));
}

/// The `history` subcommand should run successfully (with no snapshots)
#[test]
fn history_command_runs() {
    let mut cmd = Command::cargo_bin("packagist-stats-tracker").unwrap();
    cmd.args(["history", "monolog/monolog", "--period", "7d"])
        .assert()
        .success()
        .stdout(predicates::str::contains("History for"));
}

/// The `history` subcommand should reject invalid period format
#[test]
fn history_rejects_invalid_period() {
    let mut cmd = Command::cargo_bin("packagist-stats-tracker").unwrap();
    cmd.args(["history", "monolog/monolog", "--period", "invalid"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("Invalid period"));
}

/// The main help should list the history command
#[test]
fn main_help_shows_history() {
    let mut cmd = Command::cargo_bin("packagist-stats-tracker").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("history"));
}

/// The `overall` subcommand should show help
#[test]
fn overall_subcommand_shows_help() {
    let mut cmd = Command::cargo_bin("packagist-stats-tracker").unwrap();
    cmd.args(["overall", "--help"])
        .assert()
        .success()
        .stdout(predicates::str::contains("overview"));
}

/// The `overall` subcommand should run successfully (with no snapshots)
#[test]
fn overall_command_runs() {
    let mut cmd = Command::cargo_bin("packagist-stats-tracker").unwrap();
    cmd.arg("overall")
        .assert()
        .success()
        .stdout(predicates::str::contains("No snapshots found"));
}

/// The main help should list the overall command
#[test]
fn main_help_shows_overall() {
    let mut cmd = Command::cargo_bin("packagist-stats-tracker").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("overall"));
}

/// The `export` subcommand should show help
#[test]
fn export_subcommand_shows_help() {
    let mut cmd = Command::cargo_bin("packagist-stats-tracker").unwrap();
    cmd.args(["export", "--help"])
        .assert()
        .success()
        .stdout(predicates::str::contains("Export"));
}

/// The `export` subcommand should run successfully (with no data)
#[test]
fn export_command_runs() {
    let mut cmd = Command::cargo_bin("packagist-stats-tracker").unwrap();
    cmd.arg("export")
        .assert()
        .success()
        .stdout(predicates::str::contains("packages"));
}

/// The main help should list the export command
#[test]
fn main_help_shows_export() {
    let mut cmd = Command::cargo_bin("packagist-stats-tracker").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("export"));
}

/// The `collect` subcommand should accept --format option
#[test]
fn collect_accepts_format_option() {
    let mut cmd = Command::cargo_bin("packagist-stats-tracker").unwrap();
    cmd.args(["collect", "--format", "json"]).assert().success();
}

/// The `history` subcommand should accept --format option
#[test]
fn history_accepts_format_option() {
    let mut cmd = Command::cargo_bin("packagist-stats-tracker").unwrap();
    cmd.args(["history", "monolog/monolog", "--format", "json"])
        .assert()
        .success();
}

/// The `overall` subcommand should accept --format option
#[test]
fn overall_accepts_format_option() {
    let mut cmd = Command::cargo_bin("packagist-stats-tracker").unwrap();
    cmd.args(["overall", "--format", "json"])
        .assert()
        .success()
        .stdout(predicates::str::contains("tracked_packages"));
}

/// The `import` subcommand should show help
#[test]
fn import_subcommand_shows_help() {
    let mut cmd = Command::cargo_bin("packagist-stats-tracker").unwrap();
    cmd.args(["import", "--help"])
        .assert()
        .success()
        .stdout(predicates::str::contains("Import"));
}

/// The `import` subcommand should require input (stdin or file)
#[test]
fn import_requires_input() {
    let mut cmd = Command::cargo_bin("packagist-stats-tracker").unwrap();
    cmd.arg("import").write_stdin("").assert().failure();
}

/// The main help should list the import command
#[test]
fn main_help_shows_import() {
    let mut cmd = Command::cargo_bin("packagist-stats-tracker").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("import"));
}
