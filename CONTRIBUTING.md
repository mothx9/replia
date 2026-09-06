# Contributing

Start with a concrete terminal interaction problem and its observable failure
behavior. Use the [documentation map](docs/README.md) to find the contract owner
before introducing types, dependencies or documents. Keep host application
semantics outside the library.

Reconcile the shared checkout and preserve unrelated work. The
[development method](docs/development.md) defines this workflow, check selection
and the evidence required in a contribution. Agent-specific mandatory rules are
in [AGENTS.md](AGENTS.md).

Keep the safe implementation crate unsafe-free. The C binding may contain only
narrowly documented boundary unsafety and must consume public Rust API. New
dependencies must explain the correctness gap in std, license compatibility,
scope and whether their types cross a public boundary.

Include the concrete problem, resulting behavior, affected contract and exact
validation commands/results. Presentation changes need byte/cell/cursor evidence
against [the reference record](docs/presentation.md); never refresh a fixture
merely to conceal a regression. C changes require external consumer and misuse
evidence, not just a successful Rust build.

Use default rustfmt formatting and retain Cargo's lint policy and lockfile.
Documentation and examples describe implemented behavior. No placeholder API,
test count or editorial update establishes a capability or a release.
