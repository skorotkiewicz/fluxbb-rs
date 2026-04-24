# Contributing to FluxBB RS

Thank you for your interest! This project is a reimplementation of the classic FluxBB forum software in Rust using Dioxus 0.7.

## Quick Start

```bash
# Clone and build
git clone https://github.com/skorotkiewicz/fluxbb-rs
cd fluxbb-rs
just build

# Run the server
just run
```

Visit `http://localhost:8080` and run the installer.

## Development Commands

| Command | Description |
|---------|-------------|
| `just serve` | Hot-reload dev server |
| `just check` | Check compilation |
| `just fmt` | Format code |
| `just build` | Build server binary |
| `just release` | Build release binary |
| `just run` | Build and run server |
| `just db-reset` | Reset PostgreSQL schema |
| `just clean` | Clean build artifacts |
| `just reset` | Full clean + DB reset |

## Project Structure

```
src/
├── main.rs          # Routes, app shell
├── data.rs          # Server functions, DB queries, auth, CSRF
├── components/      # Reusable UI components
└── views/           # Page-level components
```

## Code Style

- Run `just fmt` before committing
- Prefer `if let` inside `rsx!` blocks over early returns
- Keep `AppShell` lightweight — per-view data loading only
- Each state-changing form needs CSRF protection

## Database

- PostgreSQL 14+ required
- Schema is in `db/schema.sql`
- After any schema change, drop and recreate: `just db-reset`

## Reporting Issues

Open an issue with:
- Steps to reproduce
- Expected vs actual behavior
- Rust version (`rustc --version`)
- PostgreSQL version

## License

By contributing, you agree that your contributions will be licensed under the Apache License 2.0.
