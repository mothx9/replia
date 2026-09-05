# Contributing

This is an early Rust library. Propose a concrete terminal interaction contract
and its observable failure behavior before adding public types or dependencies.
The [architecture](docs/architecture.md) defines scope. Historical source
evidence is kept separately in [archaeology](docs/archaeology.md).

## Checks

Use the current stable Rust toolchain, with `rustfmt` and `clippy` installed.
No MSRV has been established. Record `rustc --version --verbose` and
`cargo --version` when reporting qualification; stable CI may advance.

```sh
cargo fmt --check
cargo check --all-targets
cargo test --all-targets
cargo clippy --all-targets --all-features -- -D warnings
cargo test --doc
git diff --check
```

Use default rustfmt formatting. Cargo declares the lint policy: unsafe code is
forbidden in this crate, public documentation is required, and
Clippy's standard lint group is enabled. CI treats compiler and rustdoc
warnings as errors. A later OS boundary needing unsafe operations must review
and narrow that policy explicitly with executable lifecycle evidence.

Commit `Cargo.lock` to make repository checks reproducible. It does not pin a
downstream application's dependency resolution. R1 uses focused Unicode and Linux primitive
dependencies plus a dev-only VT state oracle; reasons and license policy are in
[architecture](docs/architecture.md). Fetch the locked graph once with
`cargo fetch --locked`; all checks can then run with `CARGO_NET_OFFLINE=true`.
Foundation tests verify registry-only dependency sources and package inventory.
Core/decoder tests verify deterministic contracts, and real Linux PTY tests
verify lifecycle and display state, including failures. Child processes isolate
color environment changes without unsafe process-global environment mutation.

Changes to visible presentation must explain intentional differences against
[the reference record](docs/presentation.md), with byte/cell/cursor evidence.
Do not regenerate snapshots simply to accept a regression. New dependencies
must justify what std lacks, correctness ownership, compatible license and
whether third-party types enter the public API.

## Independence review

For each change, inspect `cargo metadata --format-version 1 --locked --offline`
and review dependency sources, crate and feature names, exported Rust symbols,
docs, examples, environment reads, and build scripts. No application repository
may become a dependency. Do not add command catalogs or application-specific
state to the library. Historical names belong only in the archaeology record;
the license retains its required copyright attribution.

A fresh checkout must build with no sibling application repositories or
application configuration. Test it with an isolated source copy or clean CI
checkout. Do not hide a missing dependency behind a developer's filesystem.

Keep changes focused, preserve other work in a shared checkout, and include the
exact commands and results in a contribution. Add tests for implemented
behavior and meaningful failure paths. Never use placeholder tests or future
feature names as capability evidence. Do not publish a crate release as part
of foundation work.
