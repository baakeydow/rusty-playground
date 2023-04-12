# core-rusty-api

> # BAAKEYDOW's rusty playground

---

## Setup

`mv env-example .env.dev && cp .env.dev .env`

## Help

> `cargo run -- -c '--help'`

## Test all

> `cargo test --workspace`

---
# rusty_lib

## Tests

> `cargo test --manifest-path=./rusty_lib/Cargo.toml -- --nocapture`

## Dev

> `cargo watch -x 'test html_to_md --manifest-path=./rusty_lib/Cargo.toml -- --nocapture'`

---
# core-rusty-api

## Dev

> `cargo watch -x 'run --bin core-rusty-api -- --dev --log-level=debug'`

## Prod

> `sh run_rusty_api.sh`
