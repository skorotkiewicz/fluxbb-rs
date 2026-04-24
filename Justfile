# Justfile
# https://github.com/casey/just

set dotenv-load

[private]
default:
    @just --list

serve:
    dx serve

check:
    dx check

fmt:
    dx fmt

build:
    dx build

release:
    dx build --release

clean:
    cargo clean

run: build
    ./target/dx/fluxbb-rs/debug/web/server

run-release: release
    ./target/dx/fluxbb-rs/release/web/server

db-reset:
    psql "$DATABASE_URL" -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public;"

install-cli:
    cargo install dioxus-cli

reset: clean db-reset
