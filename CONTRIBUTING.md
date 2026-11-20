# Contributing

## Prerequisites

The current stable Rust toolchain, with `rustfmt` and `clippy`. `pamba` builds and is tested on every platform;
`pamba-windows` builds on Windows and is an empty crate elsewhere.

## Checks

Each of these passes before a change is proposed. Warnings are errors (`.cargo/config.toml`).

```shell
cargo fmt --all --check
cargo clippy --workspace --all-targets
cargo test --workspace --release --no-fail-fast
cargo doc --workspace --no-deps
```

## Layout

| Directory | Crate | What it holds |
| --- | --- | --- |
| `crates/pamba/` | `pamba` | The loop. Its cases are `tests/it/`. |
| `crates/pamba-windows/` | `pamba-windows` | The Windows layer. |

## Cases

A rule of the loop is held by a case in `crates/pamba/tests/it/`, named for the rule it holds. A case that waits on
another thread waits with a bound, so a broken rule fails at an assertion and never hangs the suite.
