# Elogbook

Rust backend and Yew/WebAssembly frontend for a simple electronic logbook.

## Layout

- `backend/`: Actix Web API, MongoDB access, and attachment downloads
- `frontend/`: Yew frontend built with Trunk

## Development

Create `backend/.env` from `backend/.env.example`, then run:

```bash
cargo run -p elogbook
```

In another shell:

```bash
cd frontend
trunk serve --address 127.0.0.1 --port 8081
```

Useful checks:

```bash
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cd frontend && trunk build
```
