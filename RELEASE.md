# Release process

This document describes how to release a new version of `packagist-stats-tracker` to [crates.io](https://crates.io).

## Release steps

1. **Update the version** in `Cargo.toml`:
   ```toml
   version = "0.1.4"
   ```

2. **Update `CHANGELOG.md`** with the new version and a summary of changes.

3. **Commit your changes**:
   ```bash
   git add Cargo.toml CHANGELOG.md
   git commit -m "dist: ship version 0.1.4"
   git push origin main
   ```

4. **Wait for CI to pass** — Ensure the CI workflow (fmt, clippy, tests, build) passes on your commit.

5. **Create and push a version tag**:
   ```bash
   git tag v0.1.4
   git push origin v0.1.4
   ```

## What happens next

When you push a `v*` tag, the [Release](.github/workflows/release.yml) GitHub Actions workflow automatically:

1. Checks out the code at the tagged commit
2. Verifies that the tag version matches the version in `Cargo.toml`
3. Runs `cargo publish` to publish the package to crates.io

If the tag version does not match `Cargo.toml`, the workflow will fail with an error message.
