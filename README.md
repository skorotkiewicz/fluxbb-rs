# FluxBB RS

A modern reimplementation of [FluxBB](https://fluxbb.org) — the classic lightweight forum software — built with **Rust**, **Dioxus 0.7**, and **PostgreSQL**.

> Fast. Simple. No bloat.

---

## Features

- **Full forum stack** — categories, forums, topics, replies, pagination
- **User system** — registration, login, profiles, password change
- **Moderation** — close/sticky/move/delete topics, edit/delete posts
- **Admin panel** — board settings, user management, bans, groups & permissions
- **Search** — full-text search across topics and users
- **Anti-spam** — flood protection (30s rate limit), CSRF tokens, ban system
- **Security** — bcrypt password hashing, session-based auth with CSRF double-submit
- **Responsive** — clean Air theme, works on desktop and mobile

---

## Stack

| Layer | Technology |
|-------|-----------|
| Frontend | Dioxus 0.7 (Rust → WASM) |
| Backend | Dioxus Fullstack / Axum |
| Database | PostgreSQL (SQLite planned) |
| ORM | SQLx |
| Styling | Custom CSS (Air theme) |

---

## Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) + `cargo`
- [Dioxus CLI](https://dioxuslabs.com/) — `cargo install dioxus-cli`
- PostgreSQL 14+

### 1. Clone & build

```bash
git clone https://github.com/skorotkiewicz/fluxbb-rs
cd fluxbb-rs
dx build --release
```

### 2. Run

```bash
./target/dx/fluxbb-rs/release/web/server
```

Visit `http://localhost:8080` and run the installer.

> The installer will create a .env file with your database credentials."
---

## Development

```bash
# Hot-reload dev server
dx serve

# Check compilation
dx check

# Format code
dx fmt
```

---

## Architecture

```
src/
├── main.rs          # Routes, app shell
├── data.rs          # Server functions, DB queries, auth
├── components/      # Reusable UI components
└── views/           # Page-level components (index, forum, topic, admin...)
```

- **Per-view data loading** — each page fetches only what it needs
- **Server functions** — all state changes go through type-safe Rust endpoints
- **Context API** — shared auth state across components

---

## License

MIT
