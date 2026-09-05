# Initial reference presentation: compatibility evidence

This is an engineering archaeology record, not product vocabulary. R1's visual
reference is the linear YVEX console at
`3a6520945a5c103365178f48104f0ccdb5154624` (branch `models1`, observed 2026-09-05).
The expected R0 donor was `cb336ad60c12d6fa841dc0715bba9d44aa721846`.
The intervening commits changed source/runtime work, with no changes to the
inspected console editor, palette, stream renderer, completion adapter or PTY
script. No donor source was modified or linked into REPLIA.

## Record derivation

The compact [presentation.tsv](../tests/fixtures/presentation.tsv) contains ten
named hex byte streams, 285 bytes of decoded terminal output in total. They
were captured through Linux PTYs from a temporary neutral C probe using the
**actual** `yvex_cli_terminal_style_get`, `repl_columns` and `repl_redraw`
functions extracted from the pinned donor. That probe lived outside this repo;
no donor implementation is shipped here. Its small driver supplied `demo` as
host label and invoked the observed presentation primitives below. It neither
linked a runtime nor exercised the full donor application.

The PTY transport had output postprocessing disabled to record explicit CR/LF
once (and avoid the donor transport's CR/CR/LF artifact). No content, cursor
commands, SGR or spacing was normalized after capture. `TERM=xterm-256color`
was used except for `dumb`; `NO_COLOR` was absent except for the deliberately
empty value in `no_color`. Neutral content is an explicit host substitution,
not a changed terminal style. The probe used this prompt expression:
`accent + "demo>" + reset + " "`.

| Record | Executed/reference operation | Intentional comparison |
| --- | --- | --- |
| `styled_prompt` | Redraw empty input | Exact initial prompt bytes and accented foreground |
| `no_color` | Same with NO_COLOR present and empty | Exact plain bytes, no reset residue |
| `dumb` | Same with TERM=dumb | Exact plain bytes; cursor protocol still used |
| `typed` | Empty redraw, then append `hello` | Text/cursor/style state |
| `left` | Redraw `hello` at byte cursor 4 | One-cell backward cursor motion |
| `history` | Redraw replacement `earlier` | Complete old draft replacement |
| `paste` | Empty redraw, `hello`, one CR/LF plus `... `, `world` | Intended single-newline continuation rhythm |
| `clear` | Clear visible screen/home, redraw `hello` | Explicit Ctrl-L behavior |
| `interrupt` | Redraw `hello`, append `^C` and CR/LF | Visible interrupt line |
| `resize` | Redraw `draft` | Short-input redraw after dimensions change |

Paste and interrupt records combine the actual redraw helper with the literal
control emission at `client.c:1116` and `client.c:1137`; they do not claim a full
live donor input replay. REPLIA's paste test sends CRLF, which intentionally
normalizes to the single logical newline represented here. Donor CRLF doubling
is a deficiency, not parity authority.

Pinned source evidence:
[palette and disable rules](https://github.com/yailabs/yvex/blob/3a6520945a5c103365178f48104f0ccdb5154624/src/cli/io/out.c#L236),
[redraw](https://github.com/yailabs/yvex/blob/3a6520945a5c103365178f48104f0ccdb5154624/src/cli/io/client.c#L983),
[input, continuation and interrupt](https://github.com/yailabs/yvex/blob/3a6520945a5c103365178f48104f0ccdb5154624/src/cli/io/client.c#L1089),
[prompt composition](https://github.com/yailabs/yvex/blob/3a6520945a5c103365178f48104f0ccdb5154624/src/cli/io/client.c#L1552).

## Executable oracle

`cargo test --test pty` launches separate processes with isolated environment
values and actual PTYs. It types the representative input through REPLIA's public
API, changes real PTY dimensions for resize, and compares terminal **cell text,
foreground, weight, default background and cursor** against these byte records
using the independent `vt100` parser. Initial styled/plain prompts also require
exact bytes. No OCR, runtime service, font screenshot or application repository
is needed. Only Rust test-harness chatter outside bracketed-paste lifecycle
markers is excluded from capture. Redraw's extra safe cursor/erase operations
are compared by resulting state, not removed from the stream.

Further PTY assertions cover mixed-width cursor edits, full-width row boundaries,
multiline continuation, tall drafts, height changes and external output with a
non-end cursor. Those are corrected structural contracts with explicit expected
cells; no broken donor snapshot is used as their oracle. There is no donor
active-draft external-output equivalent to copy: the new transaction is qualified
against exact preserved text/cursor and expected visible notice/draft states.

Unit tests require all seven exact SGR sequences from the reference palette and
all disable conditions. PTY cell assertions require the default background and
absence of alternate screen. The normal renderer never sets a background; its
clear commands use the terminal's own background. Ctrl-L is the explicit
full-visible-screen clear action, not a background paint or TUI switch.

## Deliberate differences and qualification limit

Grapheme movement, Unicode cells, multirow erase, viewport layout, proper resize,
CRLF normalization, atomic paste rejection and draft return repair source-derived
deficiencies. Ctrl-D on nonempty text performs forward deletion. Ctrl-C is shown
at the end of the editing surface so it cannot overwrite a draft when the cursor
is in its middle. Admission, repeated-interrupt exit, command lookup and application
labels are host choices. Arbitrary ANSI output is excluded from the safe text API.

This is evidence for the specified line-oriented visual/interaction grammar and
its corrected structural cases. It is not pixel/font equivalence, a full product
runtime transcript comparison or certification of every emulator's emoji widths,
ambiguous-width settings, saved scrollback reflow or terminal multiplexers.
