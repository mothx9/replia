# Project status

This is the sole owner of current scope, demonstrated boundaries and next work.
Contracts live in [the documentation map](docs/README.md); completed changes
live in [the changelog](CHANGELOG.md) and Git. Documentation checks establish
document consistency, not terminal correctness or consumer readiness.

## Demonstrated foundation

| Boundary | Evidence and qualification scope |
| --- | --- |
| Standalone safe Rust implementation | Deterministic grapheme editing, input, history and lifecycle tests; registry-only dependency guard |
| Linux terminal interaction | Real PTY restoration, paste, resize, interrupt/EOF and display-state assertions |
| Shared presentation | One renderer; [pinned reference and explicit differences](docs/presentation.md) |
| Pre-release C ABI 1 | Generated declarations, executed C/Rust layout probes, isolated installed static/shared consumers, misuse/FD/memory checks |
| External consumer integration | Owned and qualified by each consumer against an exact revision; not established by this library's CI |

The independent repository, editor kernel and externally consumable Rust/C
boundaries are established. The earlier lifetime-bound Rust terminal was replaced
by movable `Interaction` ownership. The repository is pre-release; Cargo
publication is disabled and no stable API, ABI or SemVer promise is made.

## Current limits

Qualification is Linux-specific. Other operating systems, an MSRV, every
emulator's grapheme width/reflow and production maturity are unqualified.
There is one active terminal interaction per linked library image, no implicit
signal policy, and no support for independent concurrent writers. Output while
editing is a synchronous safe-text line transaction. Detailed conditions belong
to [interaction](docs/interaction.md) and [presentation](docs/presentation.md).

## Next work

The agreed sequence is a separately authorized second consumer integration,
followed by a cross-consumer legacy-removal audit. Those tasks belong to their
consumer repositories and must reconcile their current evidence first. They do
not authorize library feature expansion, automatic pin advancement or a release.
An integration may justify library changes only with a reproducible generic
contract gap. No next phase starts automatically when documentation is updated.
