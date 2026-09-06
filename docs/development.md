# Development method and qualification

[Contributing](../CONTRIBUTING.md) is the entry point; [AGENTS.md](../AGENTS.md)
contains mandatory agent rules. This document owns the operational workflow.

## Reconcile and define the change

Inspect branch, HEAD/tree, remotes, status and both staged and unstaged diffs
before edits and again before commit. Establish which paths are concurrently
owned. Preserve unrelated work and never reset the checkout to fit an expected
SHA. If a relevant source changes during qualification, reconcile it and rerun
the affected checks before making a claim about the new tree.

Describe the observed problem, its owner, intended behavior and rejection paths.
Use source, tests and the ABI schema as authorities. Choose a small change that
answers a demonstrated integration or correctness need. Host policy, language
evaluation, command catalogs and application storage remain outside this library.

## Tools

Use current stable Rust with rustfmt and Clippy. No MSRV is established; record
`rustc --version --verbose` and `cargo --version` with qualification evidence.
Fetch the locked graph with `cargo fetch --locked`; Rust checks may then use
`CARGO_NET_OFFLINE=true`. Commit Cargo.lock for reproducible repository checks.

The documentation lane uses Python 3, Node 22 and a separate locked parser
installation (Mermaid 11.17.2 and jsdom 26.1.0):

```sh
npm ci --prefix tools/docs --ignore-scripts
```

These are documentation-only dependencies. They are not required by Cargo or
installed Rust/C consumers. Mermaid parses actual fenced diagrams using a DOM
provided by jsdom, without a browser, SVG copies or screenshot comparison.

Complete native qualification additionally requires cc/c++, pkg-config,
Valgrind and a Linux kernel with Landlock. Missing memory/isolation tools fail
the gate. Tests use temporary prefixes and never install into system directories.

## Select checks by boundary

| Change | Required evidence |
| --- | --- |
| Markdown, diagrams or documentation checks | Documentation checks below; manual authority/reference review |
| Rust API examples or package inventory | Documentation checks plus doctests and foundation tests |
| Editing/input/lifecycle/presentation | Full Rust checks including real PTYs; C regressions if shared behavior changes |
| C declarations, binding or staging | Generated drift check and complete isolated native qualification |
| Dependency or shared boundary changes | Full qualification and license/source/public-type review |

For the documentation surface:

```sh
python3 tools/check_docs.py
python3 -B tools/test_check_docs.py
python3 tools/generate_abi.py --check
cargo test --doc
cargo test --test foundation
git diff --check
```

The checker validates local links/anchors, document and asset reachability,
retired-surface absence, the designated project-status owner, ABI identity/tag
tables against the schema, and Mermaid syntax. Its negative fixtures must prove
that rejected documents produce a file-specific error. It does not infer prose
accuracy, verify live external URLs or prove capability by counting tests.

For Rust behavior:

```sh
cargo fmt --check
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --doc
cargo test --workspace --release
```

The single full qualification entry point is:

```sh
python3 tools/qualify.py --work /tmp/replai-qualification
```

Choose a fresh work directory. It records each command and gate log, executes
documentation/Rust checks and [native C qualification](c-api.md#executable-authority-and-current-limits),
then requires a clean repository. `--allow-dirty` supports development only;
it does not qualify clean closure. CI separates documentation, Rust core/PTY,
release, and C prepare/static/shared/memory/audit gates.

## Evidence and independence

Report the tested revision/tree, toolchain, exact commands and observed property:
termios before/after, submitted bytes, cells/cursor, FD ownership, ABI layout,
staged loader resolution or memory-checker results as appropriate. Test count,
successful compilation and a push are not substitutes for those observations.
Keep raw logs outside Git; retain small enduring fixtures only when they define
a contract. Report local evidence and remote CI separately.

Review dependency sources, crate/features, public names, environment reads,
examples and build scripts. A clean checkout must work without any application
repository or private configuration. Foundation tests enforce dependency-source
and package inventory rules. The native qualification denies repository reads
with Landlock when compiling and running staged consumers. No stale local or
globally installed library may stand in for the qualified artifact.

For ABI changes edit [api/c-abi.json](../api/c-abi.json), run
`python3 tools/generate_abi.py`, and commit generated header, Rust records,
signature assertions and probes together. A changed declaration needs layout,
misuse and real external C evidence. Never weaken core unsafety policy to make
a binding convenient.

## Documentation lifecycle

Admit a document only when a distinct subject, audience or contract needs an
owner. [The map](README.md) assigns those owners. README gives orientation;
architecture gives structure; contracts give behavior; this method gives
procedures; ROADMAP alone gives current project state. CHANGELOG records consumer
capabilities, compatibility and fixes, not editorial or implementation steps.

Transfer surviving facts before retiring a report. Preserve chronology in Git,
with an immutable link where useful, rather than archive folders or duplicate
ledgers. Keep donor names in dedicated historical/reference evidence and the
required license attribution; public API and generic examples remain neutral.
Keep diagrams embedded as Mermaid and edit their surrounding explanation with
them. Review the exact claim supported by each primary reference: a classical
evaluator description does not establish terminal conformance or API compatibility.

Before delivery inspect the actual final diff, rerun affected checks, commit
only the intended paths and use an ordinary push. Verify the remote revision
and actual CI outcomes. Report any remaining failures with causal scope rather
than either hiding them or promoting unrelated failures into library defects.
