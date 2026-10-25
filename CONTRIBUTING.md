# Contributing

Thank you for your interest in contributing to **packagist-stats-tracker**.

## Code Quality

This project uses [Just](https://just.systems/) to manage common development commands. Install Just first, then use the
recipes below.

See all available tasks with:

```bash
just --list
```

### Check (recommended)

Run formatting, linting, and tests in one command:

```bash
just check
```

### Formatting

Format all source code:

```bash
just fmt
```

### Linting

Check for common mistakes and improve Rust idioms:

```bash
just clippy
```

### Testing

Run the full test suite:

```bash
just test
```

### Building

Build a debug binary:

```bash
just build
```

Build an optimized release binary:

```bash
just build-release
```

## Pull Requests

Please open pull requests against the `main` branch from a dedicated feature branch. Use descriptive branch names that
reflect the change, such as:

- `feat/history-command`
- `fix/daily-downloads`
- `docs/update-readme`

## Commit Messages

Please follow [Conventional Commits](https://www.conventionalcommits.org/) for all commit messages. This keeps the history readable
and enables automated changelog generation.
