# R0 engineering archaeology

## R1 reconciliation and boundary refinement

At R1 entry, REPLAI was clean on `master`, HEAD
`cabffdf8c7c046857d3f4d8a4ad91f81c255824b`, tree
`df4b7ec4e269d0387ce0c04e36a9b6d00ca983d1`. Its parent remains the independent
R0 root `39bc1c7b2f630a9fd93180e73c6008244f402ed0`. The remote is the owner's
personal public repository, `mothx9/replai`.

The planned YVEX HEAD/tree were `cb336ad60c12d6fa841dc0715bba9d44aa721846` /
`528dcb1d9044d06807be2b67eb51fbd84fc0adf8`. The observed `models1` HEAD/tree
were `3a6520945a5c103365178f48104f0ccdb5154624` /
`dcf123f87ffe753744caf4f73d2d5644fdbf39d2`. The working tree was clean;
intervening commits `bc94a319` and `3a652094` did not change `client.c`, `out.c`,
`stream.c`, the input operator adapter or `tests/repl_pty.sh`. R1's presentation
reference is the observed SHA, not an assumed old snapshot. The read-only source
hash manifest also covers Makefile and the operator/source-owner registries.

The R0 tables below remain historical evidence. R1 strengthens their generic
presentation candidate: palette, prompt grammar, spacing, continuation, line
surface and output coordination belong to REPLAI. Product labels, runtime facts,
commands and semantic rendering remain with the host. The [presentation record](presentation.md)
contains the pinned source-derived byte captures and actual PTY comparisons.

| Donor issue or choice | R1 resolution and executable evidence |
| --- | --- |
| Scalar counting and byte-oriented edits | Grapheme editor plus cell layout; `tests/core.rs`, presentation unit tests and mixed-width PTY cursor assertions |
| Current draft lost when returning from history | Text and cursor saved/restored; history tests cover recalled edits and bounded admission |
| Eight-byte / 25 ms escape assumption | Incremental 64-byte staging plus drain/idle expiry; decoder tests and delayed-byte PTY test |
| Controls act during paste, CRLF doubles, incomplete paste ambiguous | Atomic bounded paste, deliberate newline normalization, observable rejection/timeout cleanup; decoder and PTY negative tests |
| One-row erase and resize dimensions ignored | Physical row layout, old-row erase and cursor-following viewport; actual PTY width/height changes and VT state assertions |
| EOF, interruption and read failures lose distinctions | Typed events and errors; nonempty Ctrl-D forward deletion is an explicit correction |
| Cleanup errors ignored and partial initialization unqualified | Exact saved termios, fallible close and non-panicking Drop; real PTY read/write/acquisition/unwind tests |
| Global signal ownership coupled to application cancellation | No handlers/masks/threads; keyboard byte interrupt, host interrupt method, dimension polling; signal-state preservation test |
| Slash registry lookup inside editing | Host reads draft/cursor and supplies a validated replacement; no command vocabulary in the library |
| Active application output has no editable-draft transaction | Generic synchronous text-output transaction preserves draft/cursor and queued input; PTY external-output tests |
| Accented palette, default background, plain-color rules, compact prompt and continuation | Intentionally preserved in generic presentation; reference records and cell/style/byte assertions |

YAI remains consumer archaeology only: the R0 linenoise/FFI seam and I01 ownership
below are unchanged requirements. No YAI source was changed, no editor was added
there, and no I01/I02 or cognitive interlock work was reopened. Neither product
is a dependency or linked into any REPLAI test. R1 performs no consumer cutover.

## R0 source record (historical)

Historical evidence for `REPLAI.RECONSTRUCTION.BASELINE.0`, inspected on
2026-09-05. Donor names in this record identify provenance and consumer
boundaries only. No source code or Git history was imported into REPLAI.

## Baseline reconciliation

| Repository | Planned branch / HEAD / TREE | Observed branch / HEAD / TREE |
| --- | --- | --- |
| `yailabs/yvex` | `models1` / `cb336ad60c12d6fa841dc0715bba9d44aa721846` / `528dcb1d9044d06807be2b67eb51fbd84fc0adf8` | Identical, local Git objects and GitHub branch checked |
| `yailabs/yai` | `master` / `82287cf92b8a95b70d387ca759b56c593848983c` / `aa3a8d8d746eb63ff61ffab599404d51fa284232` | Identical, GitHub branch and commit/tree checked |

YVEX's commit is `feat: establish multipart model-set foundation`. Its local
worktree at `/home/dgmothx/lab/yvex` has concurrent uncommitted changes. R0
reads an immutable `git archive` of the observed HEAD outside that worktree.
The inspected working-tree registry delta adds `--clear-stale-locks` to a
model download command; it does not change terminal ownership. The initial
status, index digest and tracked/untracked source digests were captured outside
both repositories for comparison. Concurrent work subsequently changed the
index/status and source/model/server files; the inspected terminal files and
PTY test stayed byte-identical. The ownership-table additions concern model
and graph owners, not the console. R0 issued no donor writes, build commands,
Git mutations or cleanup. Unchanged whole-worktree status is not claimed.

YAI's commit is `feat: add Case-owned multipart conversation content`. No
local YAI checkout was found under the user's home directory. R0 reads the
actual repository through GitHub: branch/commit metadata and a source archive
at that exact revision, extracted outside REPLAI. This does not create or
mutate a YAI worktree, branch, index or remote. YAI is consumer evidence only;
its vendored terminal implementation is not a second implementation donor.

The branches had not advanced at initial inspection. The observed YVEX source
and `ROADMAP.md:105` establish explicit `yvex chat` as the sole linear console;
bare `yvex` prints help. Older TUI findings are not current evidence.

## Source key and test evidence

All line numbers below refer to the pinned revisions above:

- [C: `src/cli/io/client.c`][C]: `client.runtime_lane`, subsystem `client`.
  This file mixes terminal mechanics and protocol-client semantics.
- [O: `config/source_owners.tsv`][O]: canonical source ownership, especially
  lines 137–175. The `generic` column means generic *within the donor* and
  does not establish suitability for an independent library.
- [P: `tests/repl_pty.sh`][P]: PTY characterization over a fake local protocol
  host; [M: `Makefile`][M], lines 723–725, owns `make test-repl`;
  [Q: `config/qa/registry.json`][Q], entry `integration.repl`, registers it.
- [I: `src/cli/input/operator.c`][I]: `cli.input.operator`, registry-driven
  argument and slash parsing. Other files in `src/cli/input/` parse product
  targets, source, graph and backend arguments, not terminal bytes.
- [S: `src/cli/io/stream.c`][S]: `cli.io.stream`, incremental rich-text and
  typed-channel rendering. Its output Unicode handling is separate from the
  input editor's weaker width calculation.
- [F: `src/cli/io/out.c`][F]: output, style, command discovery and metrics;
  [V: `src/cli/render/runtime.c`][V]: typed runtime presentation;
  [G: `config/operator/registry.json`][G]: product command authority.
- [A: `src/cli/io/content.c`][A]: `client.content_stage`, attachment lifecycle;
  [L: `src/cli/io/model.c`][L]: loaded-model selection, including a separate
  cooked `fgets` choice prompt at lines 317–336.

R0 inspected source and test assertions; it did **not** execute donor tests or
load a runtime. P asserts no alternate screen/cursor hiding, paired paste
escapes, explicit TTY refusal, command completion, paste, recall, Home/End,
resize signal, interrupt, reconnect, output during active work, cancellation
and nonempty Ctrl-D exit. Several checks only search transcript substrings.
They do not measure actual termios equality, terminal cell positions, every
key path, changed window dimensions, allocation failure or delayed escapes.
These gaps become independent R1 tests, not assumed donor passes.

## YVEX behavior classification

**A** = generic terminal mechanism; **B** = generic interaction mechanism;
**C** = product semantics; **D** = implementation artifact or deficiency.
Mixed rows split ownership explicitly. “Intended” means an interaction worth
preserving, not a claim of complete existing implementation. Deficiencies
below are source-derived unless a test assertion is identified.

| Behavior | Source evidence | Current owner | Class | Candidate REPLAI ownership | R1 test requirement | Notes / intended contract / deficiency |
| --- | --- | --- | --- | --- | --- | --- |
| TTY detection | C:1719–1723; P non-TTY scenario | Client chat dispatch | A + C | Capability check before acquisition | Pipes, redirected output, failed acquisition produce no escape output | Refusal text and exit code 2 are host policy |
| termios lifecycle; raw/cooked restoration | C:1093–1105, 1222–1235 | Client editor | A + D | Capture, scoped change and exact restore | Compare PTY termios before/after submit, EOF, interrupt, read/write failure and partial open | Intended restoration; not full raw mode (`ISIG` remains); restore/write return values ignored |
| Input byte decoding | C:1111–1143, 1212–1220 | Client editor | A + B + D | Incremental decoder and valid editing input | Split UTF-8 at every boundary; malformed/truncated bytes; embedded NUL | Input inserts arbitrary bytes; no full UTF-8 validation; NUL conflicts with later string operations |
| UTF-8 cursor movement | C:969–982, 1185–1192 | Client editor | B + D | Defined Unicode editing boundaries | ASCII, multibyte, combining sequences and joined emoji at start/middle/end | Skips continuation bytes; not grapheme-aware; no validity proof |
| Display width | C:983–1000; S:163–188, 213–243 | Client editor / stream renderer | A + D | Width policy for editor cells | Wide characters, combining marks, tabs, styled prompts and wrapping | Scalar-like count is not terminal width; output renderer is a separate product subsystem |
| Buffer insertion/deletion and limits | C:1002–1020, 1044–1072; constants 33–34 | Client editor | B + D | Bounded edits with explicit failure | Empty/full buffers, midline insertion, deletion boundaries, capacity errors preserve state | 65,536-byte limit is an artifact; initial memmove includes an uninitialized terminator byte; error collapses into loop exit |
| History storage/admission | C:939–960, 1611 | Client chat and history | B + C + D | In-memory history mechanism | Empty, duplicate, eviction and independent instances; explicit host admission | 64 entries, adjacent dedupe, silent allocation failure; only ordinary submitted text admitted; no persistence |
| Up/Down history navigation | C:1173–1184; P recall scenario | Client editor | B + D | Navigation plus saved current input | Recall boundaries and return to pre-recall draft | Down beyond newest restores empty text, losing the unsubmitted draft |
| Left/Right arrows | C:1185–1192 | Client editor | B | Cursor operations | Repeated movement at both ends, multibyte and midline edits | Intended navigation; Unicode deficiency as above |
| Home/End and Ctrl-A/Ctrl-E | C:1150–1154, 1193–1206; P Home/End | Client editor | A + B | Key decoding and start/end movement | CSI, SS3 and numeric aliases; empty and multiline text | Existing semantics are whole-buffer start/end; decide logical-line behavior explicitly |
| Backspace / Ctrl-H / Delete | C:1064–1072, 1163–1166, 1207–1209 | Client editor | A + B + D | Distinct backward/forward editing operations | No-op edges, multibyte/grapheme deletion, pasted control bytes | Backspace/DEL still edits while paste is active |
| Ctrl-L | C:1161–1166 | Client editor | A + B | Explicit clear/redraw operation | Restore draft/cursor after clearing; preserve normal scrollback behavior | Whole-screen clear is explicit input action, not the default redraw |
| Escape sequence parsing | C:1073–1088, 1168–1210 | Client editor | A + D | Bounded incremental terminal protocol | Every fragmentation boundary; lone ESC, timeout, unknown/oversized sequences | Eight-byte buffer and 25 ms per byte; unknown or delayed bytes can be dropped/misinterpreted |
| Bracketed paste | C:1105, 1171–1172, 1223; P paste-mode assertions | Client editor | A + B + D | Mode lifecycle, framing and literal payload policy | Split start/end markers, embedded controls, incomplete paste, size bound and restore | Payload ESC is parsed as commands/framing rather than generally preserved safely |
| Multiline paste / Enter | C:1134–1143; P multiline UTF-8 paste | Client editor | B + D | Atomic paste into buffer; explicit submit | LF/CR/CRLF normalize by contract; no submit before closing paste and Enter | CR and LF each become LF; CRLF can double; continuation prompt is hardcoded `... ` |
| Prompt display | C:992–1000, 1552–1562 | Editor mechanism / chat label policy | A + C | Render host-provided prompt with known width | Styled/empty/wide prompt, long input, prompt change | Model name/disconnection suffix and style choice stay with host |
| Redraw / scrollback | C:992–1000; P `assert_linear_terminal` | Client editor | A + D | Line-oriented display and cursor restoration | Emulated cell assertions for wraps, multiline buffers, no alternate screen | Clears one row and moves horizontally; no old-row/multiline geometry |
| SIGWINCH | C:961–968, 1122–1127, 1503–1509, 1648–1649; P resize | Client chat + editor | A + D | Host-compatible resize delivery and redraw | Actually change PTY size, preserve input/cursor, restore handlers | Dimensions queried then unused; P sends signal but does not resize dimensions |
| Ctrl-C detection while editing | C:961–968, 1112–1121, 1565–1567; P idle interrupt | Client signal/editor loop | A + B + C + D | Distinct interrupt delivery | Single/repeated interrupts, later resize and next input, no process exit | First clears current line; repeated exit policy is host-owned; interrupt count reset occurs only after submit |
| Ctrl-D and transport EOF | C:1130–1133, 1145–1149; P nonempty EOF | Client editor/chat | A + B + C + D | Distinct EOF/key/error outcome | Empty/nonempty Ctrl-D and read EOF/error remain distinguishable | Donor exits even with nonempty draft; test explicitly encodes that host choice; read errors also look like EOF |
| Completion invocation | C:1021–1043, 1155–1159 | Client editor with registry coupling | B + C | Host completion request at text/cursor | Zero/one/many results, cancellation/failure, arbitrary non-slash input | Tab invokes only slash-prefix lookup; ambiguity silently consumed; no-match bell |
| Completion replacement | C:1002–1043 | Client editor | B + D | Validated replacement transaction | Range boundaries, Unicode, oversized replacement, keep draft on failure | Replaces entire line, moves cursor to end; 128-byte temporary result; cursor ignored by lookup |
| Operator descriptor lookup | C:1025–1040, 1265–1286; G | Compiled operator registry / client | C | None; host supplies candidates | Generic host fixture supplies completion without a registry | `yvex_operator_descriptor` never becomes a library type |
| Slash parsing / command execution | C:1338–1465; I:226–267; G | `cli.input.operator` + client adapters | C + D | None; submit raw text to host | Literal leading slash survives submission | Host grammar uses space/tab tokenization, not quoted shell parsing; do not copy it |
| External / streamed output | C:747–922; S:551–633; P progressive stream | Client and typed stream renderer | A + B + C | Exclusive output coordination, redraw and prompt recovery | Generic partial writes without newline; interleaving, failures, cursor/draft preservation | Protocol channels, Markdown, prose layout and metrics stay host-owned |
| Generation-time terminal ownership | C:278–296, 827–834, 899–922; P async keys | Client turn lifecycle | A + B + C | Explicit suspend/resume/output ownership | Simulated host output cannot echo editing bytes or contaminate next input | Donor deliberately disables echo and flushes queued input; no editing during active work; discard is host policy |
| Signal masking / signal thread | C:219–277 | Client active-turn cancellation | A + C + D | Only deliberate host-compatible signal integration | Prior masks/handlers preserved; initialization failure; multiple instances | SIGINT/SIGUSR1 worker is coupled to remote cancellation and retry; do not transplant global architecture |
| Suspension / termination signals | C:1098–1104, 1503–1509, 1648–1649 | Client terminal lifecycle | A + D | Explicit supported-signal lifecycle policy | Supported suspend/resume and termination paths restore state; no promise for SIGKILL | Editor retains ISIG but chat installs handlers only for SIGINT/SIGWINCH; no scoped restoration for other terminating/suspending signals is established |
| Generation cancellation | C:202–240, 899–914, 1430–1441; P cancellation | Runtime protocol client | C | Interrupt notification only | Generic interrupt observable without performing any application request | `GENERATION_CANCEL`, retries and first/second-interrupt exit codes remain YVEX |
| Session reconnection / draft retry | C:1466–1481, 1574–1607; P reconnect | Client session/runtime adapter | C + B | Generic preservation/replacement of input only | Host failure can retain supplied input; no automatic replay or reconnect | Networking, stale binding checks, retries and session identity stay YVEX |
| Session management | C:1237–1254, 1405–1430, 1645–1647; G | Client + runtime session owner | C | None | Host-only fixture; core remains independent of sessions | New/use/reset/close/attach/detach are application operations |
| Reasoning commands | C:1287–1310, 1443–1455; G:1713–1776 | Runtime policy + client | C | None | No such concepts in generic API/tests | Policy and channel support checks stay YVEX |
| Attachment staging | C:1324–1391, 1613–1633; A:58–180; P attachments | `client.content_stage` | C | None | Only literal text editing is tested in the library | File reads, classification, content identity and next-turn clearing stay YVEX |
| Runtime status / metrics / rich rendering | C:render_console_status, generation_turn; V; F:378–459; S | Typed client renderers | C | None beyond output coordination | Generic fixture writes ordinary text | Status blocks, Markdown, progress and channel styling are outside library scope |
| Model selection | L:317–376; C:chat_command | CLI model selection + runtime facts | C + A | At most a reusable line-input mechanism later | Generic prompt can submit text; no selector behavior | Separate cooked numeric prompt exists; list/filter/validation stay YVEX |

This table is a requirement source for R1, not a requirement to reproduce each
donor limit, error code, protocol spelling, signal strategy or deficiency.

## YAI consumer archaeology

| Existing surface | Exact source evidence | Owner and future seam |
| --- | --- | --- |
| Rust CLI enters the advanced local prompt | [registry.rs:1956–1980][YR]; [command_adapters.rs:100–105, 1255][YC] | `yai.prompt` selects the application adapter; four C FFI functions expose line read/free and history add/max length |
| Vendored terminal library is built today | [build.rs:5–45][YB]; [vendor/linenoise/README.md][YN] | Build compiles/archives C from `vendor/linenoise`; later Rust consumption replaces this terminal seam only after separate qualification |
| Blocking editable input and history | [provider.rs:1596–1619, 3166–3240][YP] | FFI wrapper copies returned text with lossy UTF-8 conversion and frees C allocation; prompt loop sets history limit 200; trims input and admits ordinary input to history |
| TTY, finite and piped modes | [provider.rs:3166–3192][YP] | Host supports `--once` and whole-stdin input; these application modes stay host-owned |
| Terminal mechanics already available | [linenoise.c:147–484, 564–622, 1774–1850, 1880–2055][YL] | Vendored code has Unicode movement/width logic, history, raw-mode restoration, bracketed/multiline paste with folding, key editing; inventory only, not implementation reuse or fresh qualification |
| Completion and resize limits | [linenoise.c:814, 1704, 1880–1900][YL]; [command_adapters.rs:100–105][YC] | Host binds no completion callback or multiline-mode setter; no SIGWINCH handler found in the current prompt/vendor sources. Existing library capacity is not proof of host integration |
| Interrupt / EOF / error conflation | [provider.rs:1596–1609][YP]; [linenoise.c:1915–1933][YL] | Wrapper treats every null result as `None`; Ctrl-C returns null with EAGAIN, Ctrl-D on empty returns null with ENOENT. Future host seam must distinguish outcomes; do not freeze this loss of information |
| Synchronous output and application commands | [provider.rs:handle_prompt_command, prompt_repl][YP] | Prompt read completes before `run_prompt_once`; slash commands, threads, transcript retention, provider/session policy and formatted sections remain application semantics |
| I01 draft and Turn CLI | [conversation_cli.rs:38–48, 237–326][YI]; [registry.rs:896–1001][YR] | Existing create/add-text/import/derive/show/discard/send and turn list/show are registry-backed finite commands, not another line editor |
| I01 canonical state and storage | [engine conversation.rs][YE]; [conversation_cli.rs:237–326][YI] | Case/Transition owns committed Turn; application draft, ordered content, immutable storage, derivation and provenance remain YAI. SEND commits before provider execution and is not equivalent to terminal Enter |
| I01 test evidence | [multipart characterization:20–112][YT]; [I01 report][YH] | Scripts assert commit before execution, repeated modalities/ordinals, original/derived/human provenance, reopen identity and provider failure survival; inspected, not rerun |

The consumption seam is a future Rust terminal adapter replacing the
`linenoise_read_line`/history/FFI/build boundary. Host code supplies prompt and
completion, receives text/interrupt/EOF/errors and coordinates output. It
retains all interpretation and application state. There is no automatic
consumer cutover in R0 or R1.

Duplication risks: writing a second editor alongside vendored linenoise;
mistaking its optional features for configured prompt behavior; embedding
command parsing or rich sections into the library; equating editor input with
an application draft; and equating Enter with canonical SEND. The legacy prompt
still calls `run_prompt_once`; I01's finite multipart command path remains a
separate existing surface. R0 does not redesign their eventual relationship.

Both remote branch SHAs were rechecked before publication and still matched
the planned baselines. Terminal source checks covered all files under
`src/cli/io/`, `src/cli/input/`, `src/cli/render/`, plus `src/cli/main.c`, the
operator registry, `tests/repl_pty.sh` and `Makefile`; these retained their
initial bytes while unrelated work continued.

I01 stays closed. Case ownership, draft ownership, SEND, ConversationTurn,
ContentObject, multipart content, provenance, storage and provider causality
never migrate into REPLAI. No cognitive/model interlock or I02 work is included.

[C]: https://github.com/yailabs/yvex/blob/cb336ad60c12d6fa841dc0715bba9d44aa721846/src/cli/io/client.c
[O]: https://github.com/yailabs/yvex/blob/cb336ad60c12d6fa841dc0715bba9d44aa721846/config/source_owners.tsv
[P]: https://github.com/yailabs/yvex/blob/cb336ad60c12d6fa841dc0715bba9d44aa721846/tests/repl_pty.sh
[M]: https://github.com/yailabs/yvex/blob/cb336ad60c12d6fa841dc0715bba9d44aa721846/Makefile
[Q]: https://github.com/yailabs/yvex/blob/cb336ad60c12d6fa841dc0715bba9d44aa721846/config/qa/registry.json
[I]: https://github.com/yailabs/yvex/blob/cb336ad60c12d6fa841dc0715bba9d44aa721846/src/cli/input/operator.c
[S]: https://github.com/yailabs/yvex/blob/cb336ad60c12d6fa841dc0715bba9d44aa721846/src/cli/io/stream.c
[F]: https://github.com/yailabs/yvex/blob/cb336ad60c12d6fa841dc0715bba9d44aa721846/src/cli/io/out.c
[V]: https://github.com/yailabs/yvex/blob/cb336ad60c12d6fa841dc0715bba9d44aa721846/src/cli/render/runtime.c
[G]: https://github.com/yailabs/yvex/blob/cb336ad60c12d6fa841dc0715bba9d44aa721846/config/operator/registry.json
[A]: https://github.com/yailabs/yvex/blob/cb336ad60c12d6fa841dc0715bba9d44aa721846/src/cli/io/content.c
[L]: https://github.com/yailabs/yvex/blob/cb336ad60c12d6fa841dc0715bba9d44aa721846/src/cli/io/model.c
[YR]: https://github.com/yailabs/yai/blob/82287cf92b8a95b70d387ca759b56c593848983c/cmd/yai/src/cli/registry.rs
[YC]: https://github.com/yailabs/yai/blob/82287cf92b8a95b70d387ca759b56c593848983c/cmd/yai/src/command_adapters.rs
[YB]: https://github.com/yailabs/yai/blob/82287cf92b8a95b70d387ca759b56c593848983c/cmd/yai/build.rs
[YN]: https://github.com/yailabs/yai/blob/82287cf92b8a95b70d387ca759b56c593848983c/vendor/linenoise/README.md
[YP]: https://github.com/yailabs/yai/blob/82287cf92b8a95b70d387ca759b56c593848983c/cmd/yai/src/provider.rs
[YL]: https://github.com/yailabs/yai/blob/82287cf92b8a95b70d387ca759b56c593848983c/vendor/linenoise/linenoise.c
[YI]: https://github.com/yailabs/yai/blob/82287cf92b8a95b70d387ca759b56c593848983c/cmd/yai/src/conversation_cli.rs
[YE]: https://github.com/yailabs/yai/blob/82287cf92b8a95b70d387ca759b56c593848983c/engine/yai-engine/src/conversation.rs
[YT]: https://github.com/yailabs/yai/blob/82287cf92b8a95b70d387ca759b56c593848983c/tests/characterization/multipart-conversation/test_multipart_conversation.sh
[YH]: https://github.com/yailabs/yai/blob/82287cf92b8a95b70d387ca759b56c593848983c/refoundation/foundation-recovery/interlock-01/INTERLOCK-I01-REPORT.md
