# Contributing

## Prerequisites

The Rust toolchain `rust-toolchain.toml` names, which `rustup` fetches, and `cargo-deny` and `cargo-semver-checks`.
`pamba` builds and is tested on every platform; `pamba-windows` builds on Windows and is an empty crate elsewhere.

## Checks

Each of these passes before a change is proposed, on Linux for the loop and on Windows for the Windows layer. They are
the commands the project's gate runs.

```shell
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --locked
cargo package --workspace --locked
cargo deny check
cargo semver-checks
```

## Layout

| Directory | Crate | What it holds |
| --- | --- | --- |
| `crates/pamba/` | `pamba` | The loop. Its cases are `tests/it/`. |
| `crates/pamba-windows/` | `pamba-windows` | The Windows layer. |

## Cases

A rule of the loop is held by a case in `crates/pamba/tests/it/`, named for the rule it holds. A case that waits on
another thread waits with a bound, so a broken rule fails at an assertion and never hangs the suite.
