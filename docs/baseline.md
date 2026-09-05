# Baseline evidence

The R0 sections below are historical. R1 qualification is recorded at the end.

Wave: `REPLIA.RECONSTRUCTION.BASELINE.0`. Date: 2026-09-05.
This record covers the foundation before its genesis commit. Git and the final
delivery report own the resulting commit/tree and publication status; this
file does not attempt to contain the hash of its own commit. The publication
correction below was recorded after genesis at the owner's explicit request.

## Environment and creation

The canonical local path `/home/dgmothx/lab/replia` was absent. Authenticated
GitHub lookup reported that `yailabs/replia` did not exist, including a second
check before publication preparation. A new directory was created with
`mkdir`, followed by `git init -b master`. No repository template, fork, clone,
subtree, copied Git directory or donor source import was used.

During publication the owner corrected the destination to the personal
profile. After verifying `mothx9/replia` was absent, the existing independent
repository was transferred to that account. GitHub confirmed public ownership
by `mothx9`, default branch `master`, and the same genesis commit. The canonical
remote is now **https://github.com/mothx9/replia**. A subsequent ordinary commit
updates the package URL and this provenance record; history was not rewritten.

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

## R1 kernel qualification

Wave: `REPLIA.TERMINAL.EDITOR.KERNEL.0`, 2026-09-05. Started on clean `master`
at `cabffdf8c7c046857d3f4d8a4ad91f81c255824b`, tree
`df4b7ec4e269d0387ce0c04e36a9b6d00ca983d1`; continued on that branch without
reset, worktree or history rewrite. The personal public remote remains
`https://github.com/mothx9/replia`. The resulting commit/tree are owned by Git
and the delivery report, not a self-referential hash in this document.

The observed donor and planned baseline are recorded in [archaeology](archaeology.md).
Eight inspected donor file hashes remained unchanged, and the live donor tree
remained clean. The R0 YAI archive (1353 regular files) was compared byte-for-byte
with its source snapshot and remained identical. No YAI local Git checkout was
created; that archive is read-only archaeological evidence, not a dependency.

Actual qualification environment: `rustc 1.98.1 (48a229cea 2026-09-01)`,
`cargo 1.98.1 (797e8a9bc 2026-08-05)`, host `aarch64-unknown-linux-gnu`, LLVM
22.1.8, Linux `6.17.0-1021-nvidia`. No MSRV or non-Linux qualification is inferred.
Checks used `CARGO_NET_OFFLINE=true`, `RUSTFLAGS='-D warnings'` and
`RUSTDOCFLAGS='-D warnings'` after fetching the locked registry dependencies.

| Exact command | Result |
| --- | --- |
| `cargo fmt --check` | Exit 0 |
| `cargo check --all-targets` | Exit 0 |
| `cargo test --all-targets` | Exit 0; 32 tests passed, none failed/ignored |
| `cargo clippy --all-targets --all-features -- -D warnings` | Exit 0; warning-clean |
| `cargo test --doc` | Exit 0; two doctests passed |
| `git diff --check` | Exit 0 |
| `cargo build --example demo` | Exit 0; executable then exercised on a real PTY |

The all-target total comprises ten internal decoder/layout/lifecycle tests,
six public core tests, two packaging/dependency guards and fourteen PTY harness
tests. One PTY test is the environment-isolated child entrypoint; the reference
parent invokes it for ten independent recorded cases. The example test harness
itself has zero unit tests; its executable was separately driven through echo,
unique completion, multiline paste, notice during a mid-cursor draft, interrupt
and EOF, with exact termios restoration asserted. No terminal capability is
inferred from an empty example harness.

Negative coverage includes invalid/non-TTY/mismatched handles, unsupported
window size, partial acquisition, a real write-only input FD/read error, a real
read-only output FD/write error **after** acquisition, unwind cleanup, competing
instances, invalid/incomplete UTF-8, unknown/fragmented sequences, overflow,
incomplete paste, pasted controls, invalid completion/output text and unchanged
signal state. Display coverage includes mixed cell widths, continuation, exact
row edges, tall drafts, actual width/height changes, non-end cursors, queued
input retained across output, all palette roles and environment disable rules.
The [presentation record](presentation.md) states exactly which donor-derived
byte/state comparisons were executed and their limits.

Dependency manifest audit:

| Direct dependency | Role / reason std is insufficient | License selection | Public type leakage |
| --- | --- | --- | --- |
| `rustix 1.1.4` | Safe termios, FD identity and poll; PTY acquisition only in tests | MIT option | None; API uses std FDs/errors |
| `unicode-segmentation 1.13.3` | Extended grapheme boundaries absent from std | MIT option | None |
| `unicode-width 0.2.2` | Unicode cell-width tables absent from std; normal ambiguous-narrow policy | MIT option | None |
| `vt100 0.16.2` (dev only) | Independent ANSI/VT cell/style/cursor oracle absent from std | MIT | None |

The locked graph has one workspace root and fourteen registry packages; all
transitive packages offer MIT. `cargo tree` tests reject any extra local/Git
source. No package features, custom build script, application dependency, native
consumer link, C ABI or release was added. Public API, examples, style roles,
environment reads and package inventory were manually audited. The only library
environment variables are `NO_COLOR` and `TERM`. Existing MIT attribution remains.

All five Cargo commands also passed in
`/tmp/replia-r1-isolated-ve3ssd8i/replia`, a fresh source copy and target directory.
A private Cargo home held only a copied registry cache. Linux Landlock denied
read/execute access to source outside the copied crate, dependencies, toolchain
and system paths. Access probes explicitly returned `PermissionError` for the
original crate manifest, the donor editor and the consumer archive's manifest
before offline compilation and all PTY tests ran. Neither source archive was
renamed, removed or changed. This proves the build and tests can operate when
both application source trees are inaccessible.

Local ephemeral evidence: `/tmp/replia-r1-37brrbog/checks.json`, `isolation.py`,
`isolation.log`, `donor.json`, `presentation_probe.c`, `demo-pty.bin`. Committed
tests and byte records carry the reproducible contracts; these temporary audit
files are not build inputs. CI fetches the locked registry graph, runs all six
authoritative checks on a clean Linux checkout with warnings denied, and checks
tracked/untracked cleanliness. The pushed commit's actual CI result is reported
only after GitHub completes that run.

R1 does not qualify other operating systems, every emulator's Unicode/reflow
behavior, unrestricted concurrent writers, termination bypassing cleanup,
unframed paste, a stable API/ABI or production maturity. No consumer was cut over,
no release published and no R2 ABI work begun.
