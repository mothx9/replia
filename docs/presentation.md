# Presentation contract and reference evidence

This document owns the terminal surface and its executable reference evidence.
The historical visual reference is the linear YVEX console at
`3a6520945a5c103365178f48104f0ccdb5154624` (branch `models1`, observed 2026-09-05).
The expected R0 donor was `cb336ad60c12d6fa841dc0715bba9d44aa721846`.
The intervening commits changed source/runtime work, with no changes to the
inspected console editor, palette, stream renderer, completion adapter or PTY
script. No donor source was modified or linked into REPLAI.

## Prompt, cells and redraw

Layout uses `unicode-width`'s normal (ambiguous-narrow) cell policy per grapheme.
ANSI style sequences are not text and contribute no cells. Draft TAB expands
to four-column stops. CJK wide characters, accented text, combining clusters,
ordinary emoji and joined emoji are covered by core/layout tests. The independent
VT oracle covers the subset it can model; a font or emulator may render a joined
emoji or ambiguous character differently. Full Unicode terminal equivalence,
bidi layout and all terminal width tables are not claimed.

Prompt fields are plain control-free text, at most 1024 bytes each. The label
and optional literal suffix compose as `<Accent>label+suffix><Default> `;
continuations default to `... `. No raw ANSI prompt injection is accepted.
Style roles are Default, Strong, Accent, Dim, Success, Warning and Error. All
SGR values have one authority in [`Theme`](../src/presentation.rs). No background color or alternate
screen is set. Non-TTY output, `NO_COLOR` present (including empty), or
`TERM=dumb` disables styling. Disabling color emits no SGR, including no reset
residue, while preserving text. `TERM=dumb` follows the reference's color rule; it is **not** a
promise to operate on a terminal that lacks the cursor/erase protocol entirely.

Physical rows are explicitly laid out with CR/LF; full-width boundaries and
wide-character gaps are handled without counting scalars as columns. A logical
newline immediately after a full row does not introduce an extra blank row.
The previous editing rows are erased before redrawing, then the logical cursor
is restored. Normal short end insertion appends directly, preserving the compact
reference rhythm. Ctrl-L explicitly clears the visible screen and redraws.
For drafts taller than the terminal, a cursor-following viewport retains at most
height minus one physical rows; hidden text is retained and still submitted.
Minimum dimensions are two columns and two rows. Resize qualification covers
PTY dimension changes and the VT cell model, not every emulator's scrollback
reflow policy; the [executable oracle](#executable-oracle) states the evidence.

## External output

`external_output(role, text)` is a synchronous display transaction: disable paste
framing, clear the editing rows, write validated host text using a generic role,
finish its line, enable paste and redraw draft/cursor. LF/CRLF are normalized;
other controls except TAB reject before any terminal mutation. There is no raw
ANSI passthrough. Input stays raw and queued bytes are retained. The host controls
when output is written; independent concurrent writes must be serialized through
this method. Partial fragments can be sent as separate lines; continuous
no-newline streaming batches and an unrestricted writer guard are not supported APIs.

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
live donor input replay. REPLAI's paste test sends CRLF, which intentionally
normalizes to the single logical newline represented here. Donor CRLF doubling
is a deficiency, not parity authority.

Pinned source evidence:
[palette and disable rules](https://github.com/yailabs/yvex/blob/3a6520945a5c103365178f48104f0ccdb5154624/src/cli/io/out.c#L236),
[redraw](https://github.com/yailabs/yvex/blob/3a6520945a5c103365178f48104f0ccdb5154624/src/cli/io/client.c#L983),
[input, continuation and interrupt](https://github.com/yailabs/yvex/blob/3a6520945a5c103365178f48104f0ccdb5154624/src/cli/io/client.c#L1089),
[prompt composition](https://github.com/yailabs/yvex/blob/3a6520945a5c103365178f48104f0ccdb5154624/src/cli/io/client.c#L1552).

## Executable oracle

`cargo test --test pty` launches separate processes with isolated environment
values and actual PTYs. It types the representative input through REPLAI's public
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

## The same renderer through C

The observed read-only donor at R2 start was
`5b95ee82eee394581521d106c7b1ec479d472448`, branch `models2`, tree
`7f1065cda89b12a54d81591f801f492da70594ca`. The console, palette, stream output
and PTY script are unchanged from the R1 oracle revision above. R2 retains that
record instead of following unrelated runtime work.

`tools/c_pty.py` drives the **external C process** built from an installed header
and release library. It does not call Rust editor functions from the harness.
The dev-only `terminal-state` executable uses the independent vt100 parser to
report cells, foreground, bold weight, default background and cursor. Both
static/shared processes, plus the shared process under Valgrind, run the same
scenarios. Initial styled/NO_COLOR/dumb prompt bytes must equal the R1 record
exactly after the bracketed-paste enable marker. All seven generic style roles
are checked in actual C output. There is only one Rust renderer beneath both APIs.

| C scenario | Observed state required by assertions (zero-based row,column) |
| --- | --- |
| C01 styled / C02 NO_COLOR / TERM=dumb | `demo> `; cursor (0,6); Accent 81 or default; exact prompt bytes |
| C03 UTF-8 edit | `hé界🌍`, Left, Backspace, `X` → `héX🌍`; byte cursor 4, cells (0,9) |
| C04 history / C14 reopen | Submit `earlier`; draft `draft`, Left, Up, Down → original draft and byte cursor 4; cells (3,10) |
| C05 completion | `wor`, Tab → request with `wor`, C chooses `world`; byte cursor 5, cells (0,11) |
| C06 paste | Bracketed `é` CRLF `界` → one `é` LF `界` input; continuation `... `, cursor (1,6) |
| C07 resize | 12 to 9 columns while editing `ab界` LF `line 🌍`; four physical rows, cursor (3,0); inserting X gives `ab界` LF `line X🌍` |
| C08 external output | Dim notice above `demo> ab界`; byte cursor 2 remains at (1,8); X gives `abX界` |
| C09 interrupt | `hello`, Ctrl-C → event 2, visible `demo> hello^C`, cursor (1,0) |
| C10 empty Ctrl-D | Event 3, empty draft, cursor (1,0) |
| C11 nonempty Ctrl-D | `abc`, Ctrl-D at end, Home, Ctrl-D → `bc`, byte cursor 0, cells (0,6); then submit |
| C12 submit | `hello`, Enter → event 1 and exact echo text; cursor (3,0) |
| C13 restoration / C15 destruction | Captured termios before == after; paste disabled; destroy success; process exit 0 |
| Ctrl-L | `draft`, Left, Ctrl-L → same draft, cursor (0,10), cleared visible surface |

Every scenario checks default background and no alternate screen. Resize keeps
the real trailing space on its full-width continuation row; normalization must
not delete a visible cell. PTY JSON retains input hex, complete output bytes,
events, text and cursor observations. These are terminal-state/byte contracts,
not a claim about pixel rendering or arbitrary emulator reflow.
