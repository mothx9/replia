# R0 foundation evidence

Wave: `REPLIA.RECONSTRUCTION.BASELINE.0`. Date: 2026-09-05.
This record covers the foundation before its genesis commit. Git and the final
delivery report own the resulting commit/tree and publication status; this
file does not attempt to contain the hash of its own commit.

## Environment and creation

The canonical local path `/home/dgmothx/lab/replia` was absent. Authenticated
GitHub lookup reported that `yailabs/replia` did not exist, including a second
check before publication preparation. A new directory was created with
`mkdir`, followed by `git init -b master`. No repository template, fork, clone,
subtree, copied Git directory or donor source import was used.

Rust was initially absent from PATH. The official rustup installer installed
the current stable toolchain with the minimal profile plus rustfmt and Clippy;
shell startup files were not modified. The actual toolchain reported:

```text
rustc 1.98.1 (48a229cea 2026-09-01)
commit-hash: 48a229ceaefd4985c50990b14116b6d856af0985
host: aarch64-unknown-linux-gnu
release: 1.98.1
LLVM version: 22.1.8
cargo 1.98.1 (797e8a9bc 2026-08-05)
Linux 6.17.0-1021-nvidia aarch64
```

This is recorded evidence, not an MSRV or portability promise. Installation
followed the [official instructions](https://rust-lang.org/install.html);
the [stable release announcement](https://blog.rust-lang.org/2026/09/03/Rust-1.98.1/)
identifies the installed release.

## Authoritative checks

Working directory: `/home/dgmothx/lab/replia`. Environment:
`CARGO_NET_OFFLINE=true`, `RUSTFLAGS='-D warnings'`,
`RUSTDOCFLAGS='-D warnings'`; standard stable Cargo binaries in PATH.

| Order | Exact command | Exit / result |
| --- | --- | --- |
| 1 | `cargo fmt --check` | 0; no formatting changes |
| 2 | `cargo check --all-targets` | 0; library and test target checked |
| 3 | `cargo test --all-targets` | 0; two package contract tests passed |
| 4 | `cargo clippy --all-targets --all-features -- -D warnings` | 0; no warnings |
| 5 | `cargo test --doc` | 0; zero doctests, no operational API examples claimed |
| 6 | `git diff --check` | 0 |

Unedited test result excerpt:

```text
running 2 tests
test foundation_dependency_graph_contains_only_the_library ... ok
test distributable_contains_source_license_and_development_contract ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

The library unit harness has zero tests because no editor is implemented.
The two integration tests exercise Cargo's actual dependency graph (all
features, targets and dependency kinds) and package file listing. Neither
asserts terminal capability. `cargo package --list` only lists files; no crate
release was created or published.

## Independence proof

`cargo metadata --format-version 1 --locked --offline` reports one workspace
member/package, no dependencies, no features and no custom build target. The
only library target is `replia`; the other target is the foundation test.
`publish = false` and `build = false` are explicit manifest policy. Cargo.lock
contains one package. There are no exported Rust items, runtime environment
reads, examples, native links, headers, optional features or build scripts.
Manual review covered all those surfaces and the public documentation.

The complete source tree was copied without `.git` or `target` to
`/tmp/replia-isolated-h6838tez/replia`. A fresh Cargo home and target directory
were used. A Linux Landlock read/execute restriction allowed only that
temporary root, the Rust toolchain and system directories. Reads of the
original crate and both external source snapshots were explicitly rejected
with `PermissionError` before Cargo ran. All five Cargo commands above then
passed offline inside that restriction, including both integration tests.
The source snapshots remained on disk for archaeology but were inaccessible
to the compiler/test process. No donor path had to be renamed or removed.

Local audit artifacts (outside Git, ephemeral):
`/tmp/replia-r0-3mthawvf/local-checks.json`, `isolation.py`, `isolation.log`.
The successful isolated run uses a private `TMPDIR` and permits file renames
within allowed trees; initial sandbox probes without those settings failed
before this successful run. User/mount namespace and container probes were
unavailable on this host. These were harness setup failures, not crate passes.

CI provides a separate reproducible foundation: a clean checkout on a hosted
Linux runner, current stable rustfmt/Clippy, the same checks, offline Cargo and
a tracked-file cleanliness check. Actual workflow success is reported only
after GitHub returns the result for the pushed commit.

## Limits of this evidence

There is no terminal implementation, compatibility certification, performance
measurement, consumer integration, C ABI or published crate. All terminal
requirements in [architecture](architecture.md) remain R1 work. Historical
source classification and read-only consumer inspection are documented in
[archaeology](archaeology.md), with pinned references and test-evidence limits.
