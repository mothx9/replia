# Changelog

Externally meaningful changes are recorded here. This is not a release ledger
or an implementation diary; [project status](ROADMAP.md) owns current limits and
next work. All entries below are unreleased and carry no compatibility promise.

## Unreleased

- Introduced an independent Rust terminal interaction library with bounded
  Unicode/grapheme editing, draft-preserving history, host completion, multiline
  paste, typed outcomes, Linux terminal restoration and coordinated output.
- Added terminal-native prompt/style roles, continuation and multirow redraw,
  respecting color-disable rules and the terminal's default background.
- Replaced lifetime-bound `Terminal<'a>` with owned `Interaction` composition
  to support explicit close/reopen and movable owners. This breaks the initial
  experimental Rust composition API.
- Added pre-release C ABI 1 with explicit handle/FD/buffer ownership, staged
  static/shared artifacts and pkg-config integration through the public Rust API.
- Renamed the project, Rust package and native symbols/artifacts from REPLIA to
  REPLAI. Consumers of the previous spelling must update includes, symbols,
  package names and linkage together; no compatibility aliases are supplied.
