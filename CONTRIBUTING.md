# Contributing to Pamba

Every pull request to `main` runs the checks of
[`shuwariafrica/public-crates`](https://github.com/shuwariafrica/public-crates), which the
organisation requires on this repository; it holds no workflow of its own. A first-time
contributor's run waits until a maintainer approves it.

## The checks, as you run them

The toolchain is the one `rust-toolchain.toml` names, which rustup installs. `cargo deny` and
`cargo semver-checks` are public-crates' versions, held to a digest in its `mise.lock`:

```
git clone https://github.com/shuwariafrica/public-crates
mise -C public-crates install --locked cargo-deny github:obi1kenobi/cargo-semver-checks
export PATH="$(mise -C public-crates where cargo-deny):$(mise -C public-crates where github:obi1kenobi/cargo-semver-checks):$PATH"
```

Then, in this repository, in the order the checks run them:

```
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --locked
cargo package --workspace --locked
cargo deny check
cargo semver-checks
```

`cargo semver-checks` compares each crate with its last release on crates.io; before a crate's
first release the checks skip it and say so in the run's summary.

Before those, the checks refuse a tree:

- whose toolchain is anything but one exact release, in `rust-toolchain.toml` alone: no
  `rust-toolchain` file, no `path`, no channel such as `stable`;
- in which any publishable crate's `rust-version`, or the workspace's, is not that release;
- whose crates are not all under the one licence `LICENSE` holds;
- whose `Cargo.lock` takes a package from a git source public-crates' `projects.json` does not list
  for this project, or at a revision off the first-parent line of the branch it names.

## Changing what the checks check

A pull request fails, by the name of each file, when it changes any of:

- `.cargo/config.toml` or `.cargo/config`;
- `deny.toml`;
- `clippy.toml` or `.clippy.toml`;
- `rustfmt.toml` or `.rustfmt.toml`;
- `rust-toolchain.toml` or `rust-toolchain`;
- any file under `.github/`;
- a manifest's `[lints]` or `[workspace.lints]`, or its `rust-version`.

A tree may hold any of them; only a change fails. Such a pull request lands only by an
organisation owner's deliberate merge, which covers the configuration change alone: the checks
above must still pass.

One change passes as data: raising the toolchain. `rust-toolchain.toml` changes its `channel`
alone, from one exact stable release to a later one, and every publishable crate's
`rust-version`, and the workspace's, moves to that release in the same pull request. Dependabot's
toolchain pull request changes the channel only, so it fails until `rust-version` is moved on its
branch.
