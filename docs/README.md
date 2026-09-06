# Documentation map

Each subject has one owner. Source, tests and the ABI schema take precedence
over prose; summaries link to the owner instead of restating its contract.

| Reader's question | Authority |
| --- | --- |
| What is the library and how do I try it? | [Repository README](../README.md) |
| What is a classical REPL, and which part does REPLAI implement? | [REPL guide](repl.md) |
| Which modules and language boundaries own the implementation? | [Architecture](architecture.md) |
| What do editing, input, history, completion and lifecycle guarantee? | [Interaction](interaction.md) |
| How are prompt, cells, redraw and notices presented and tested? | [Presentation](presentation.md) |
| How does a C host install, link and use ABI 1? | [C API](c-api.md) |
| How do I propose a change? | [Contributing](../CONTRIBUTING.md) |
| How do I reconcile, validate and report a change? | [Development](development.md) |
| Which rules must a coding agent follow? | [Agent instructions](../AGENTS.md) |
| What is demonstrated, limited and next? | [Project status](../ROADMAP.md) |
| What changed for a consumer? | [Changelog](../CHANGELOG.md) |

Rustdoc owns method-level Rust API documentation (`cargo doc --no-deps`).
[The ABI schema](../api/c-abi.json) owns C declarations and generated probes;
[the header](../include/replai.h) is the installed consumer interface.

Completed genesis and reconstruction reports live in Git:
[baseline evidence](https://github.com/mothx9/replai/blob/57792794ee1ef6f460a91130e7e79e0d21b94956/docs/baseline.md)
and [source archaeology](https://github.com/mothx9/replai/blob/57792794ee1ef6f460a91130e7e79e0d21b94956/docs/archaeology.md).
Their surviving contracts are owned above. The pinned donor comparison remains
in presentation because it still defines an executable oracle.
