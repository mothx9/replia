# Contributing

This is an early Rust library. Propose a concrete terminal interaction contract
and its observable failure behavior before adding public types or dependencies.
The [architecture](docs/architecture.md) defines scope. Historical source
evidence is kept separately in [archaeology](docs/archaeology.md).

## Checks

Use the current stable Rust toolchain, with `rustfmt` and `clippy` installed.
Full C qualification also requires cc/c++, pkg-config, Python 3, Valgrind and
a Linux kernel with Landlock; missing memory/isolation tools fail the gate.
No MSRV has been established. Record `rustc --version --verbose` and
`cargo --version` when reporting qualification; stable CI may advance.

```sh
cargo fmt --check
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --doc
cargo test --workspace --release
python3 tools/qualify.py
git diff --check
```

Use default rustfmt formatting. Cargo declares the lint policy: unsafe code is
forbidden in this crate, public documentation is required, and
Clippy's standard lint group is enabled. CI treats compiler and rustdoc
warnings as errors. The separate C binding permits narrowly documented unsafe pointer/FD operations
with `unsafe_op_in_unsafe_fn = "deny"`; it must use only the public safe crate API.
Do not weaken the implementation crate policy to accommodate a binding.

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

## ABI changes

Edit `api/c-abi.json`, then run `python3 tools/generate_abi.py`. Commit the
self-contained header, Rust records/signature checks and C/Rust layout probes
alongside it. `--check` rejects drift. Any contract change needs misuse and
external consumer evidence, not merely an updated header. See [C API](docs/c-api.md).
Use a new temporary `--work` directory for qualification; staged artifacts are
never installed into system directories. Local `--allow-dirty` development runs
do not satisfy the clean-repository closure gate.
