# Contributing

## Local Checks

Run these before committing:

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
```

On Windows GNU, put `C:\msys64\ucrt64\bin` on `PATH` before Cargo commands.

## Change Rules

- Keep filesystem writes behind `lode-core::ValidatedRoot`.
- Keep child processes behind `lode-core::Process`.
- Add or update a focused test for security-sensitive behavior.
- Do not commit generated output from `target/`, `outputs/`, editor caches, or local secrets.