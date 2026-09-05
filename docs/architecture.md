# Architecture and implementation boundary

Status: `REPLIA.RECONSTRUCTION.BASELINE.0` (R0). This is the genesis of an
independent terminal interaction library. The crate currently exports no
operational API; this document defines ownership, not implemented capability.

## Ownership

| Candidate library mechanism | Host responsibility |
| --- | --- |
| Editable text, cursor invariants, insertion and deletion | Meaning, validation and execution of submitted text |
| In-memory history navigation | Which entries to retain; persistence, storage paths and retention policy |
| Input decoding, paste framing, key recognition | Application command language and keybinding consequences |
| Submit, interrupt and end-of-input delivery | Commit, cancel, retry or exit decisions |
| Completion invocation and validated text replacement | Candidates, vocabulary, eligibility and application lookup |
| Terminal acquisition, restoration, resize and redraw | When interaction begins/ends and what the prompt says |
| Coordinated output and restoration of the editing display | Content formatting, application progress and scheduling |

Terminal I/O is the only intended external effect of the library. Application
filesystem actions, durable application state, network execution, command
registries, content interpretation and authorization remain outside it.
No background worker, signal policy or process-global owner is implied by this
table. The host must be able to retain control of its application loop.

## Decomposition hypothesis

Three internal domains may become useful as implementation evidence develops:

- **Core:** deterministic text/cursor state and editing transitions, independent
  of file descriptors, clocks, signals and application objects.
- **Terminal:** OS/TTY lifecycle, byte protocol, dimensions and screen updates.
- **Interaction:** the boundary through which a host receives input outcomes,
  provides completion and coordinates external output.

These are working names, not public modules or a framework. Core must not
depend on terminal or host implementations. Terminal and interaction adapters
may use core state. Introduce modules and public types only as a concrete
contract requires them; no ABI is established by R0.

## Behavioral discipline

Preserve intended interaction, not every artifact of the reference source.
The [archaeology record](archaeology.md) distinguishes generic mechanisms,
host meaning, observed implementation choices and source-derived deficiencies.
Its source references are pinned. Existing test text is evidence of test
intent; it is not a claim that those tests were run during R0.

The following negative requirements constrain future implementation:

1. Every successful terminal state change needs paired cleanup on normal exit,
   EOF, interrupt, I/O failure and partial initialization. Restore the captured
   state, not a guessed cooked-mode default. Report explicit restoration failure;
   cleanup during unwinding must not panic. Uncatchable termination is outside
   a library's restoration guarantee.
2. Do not silently replace host signal handlers, change unrelated masks, exit
   the process, or install a global signal thread. Decide signal integration
   and exclusive terminal ownership before implementing them.
3. Keep cursor offsets on valid text boundaries. Specify invalid/incomplete
   UTF-8 handling, grapheme movement and terminal display width separately.
   A byte count or scalar count is not a column count.
4. Parse fragmented/unknown escape sequences and paste boundaries with bounded
   memory and progress. Pasted newlines must not submit multiple commands.
   Specify CR/LF normalization and treatment of pasted control bytes; payload
   must not execute terminal escapes or silently invoke editing shortcuts.
5. Redraw must preserve text, cursor and scrollback at wraps, multiple lines
   and resized widths. Prompt styling cannot be counted as visible columns.
   No alternate screen or dashboard is needed for a line-oriented editor.
6. EOF, interrupt and I/O error must be distinguishable. Deliver an interrupt
   without choosing what application operation it cancels. Buffer discard,
   empty/nonempty Ctrl-D behavior and repeated interrupt policy require
   explicit contracts; they cannot follow accidentally from transport errors.
7. Completion replacement must validate range and text boundaries and handle
   no match, ambiguity and failure without losing the existing input. No
   command grammar or slash prefix is intrinsic to completion.
8. There must be one coordinated terminal writer. Host output must not corrupt
   a draft or cursor. Whether editing is suspended, retained or resumed is
   explicit; silently discarding queued input is not a universal library rule.
9. State remains instance-owned. History admission and persistence belong to
   the host; navigation must not unexpectedly destroy its current draft.
   Limits and capacity failures must be observable and leave valid state.
10. Detect unsuitable input/output before changing terminal state. Define the
    tested Linux TTY boundary and return actionable errors; do not claim a
    portable backend or invent a non-TTY application policy.

R0 cannot qualify these requirements because it contains no editor. No feature,
performance, compatibility, C ABI or terminal portability claim follows from a
successful foundation build.

## Exact next wave

`REPLIA.TERMINAL.EDITOR.KERNEL.0` (R1) is the next bounded implementation delta:

1. Resolve the input, Unicode, limit, EOF and interrupt choices above in
   executable tests, then implement a small independent Rust editing kernel:
   buffer/cursor operations, navigation and in-memory history with draft return.
2. Add bounded incremental input/escape decoding, bracketed paste and newline
   handling; test fragmentation, invalid input and capacity failure.
3. Add the Linux terminal lifecycle and scrollback-preserving prompt/redraw
   adapter with explicit ownership. Test real PTYs, exact termios restoration,
   partial-open and I/O failure, resize, wraps and multiline input.
4. Establish only the minimal host boundary needed for submit, interrupt, EOF,
   completion request/replacement and suspend/output/redraw/resume. A generic
   fixture must prove the boundary without an application dependency. An
   asynchronous scheduler or streaming framework is not required.
5. Expose only Rust API proven by those tests; document Linux qualification
   and any remaining limitations. Revisit lint/dependency policy only with
   implementation evidence. Run all R0 checks plus kernel and PTY checks.

No consumer cutover, C ABI, application command registry, rich-content renderer,
panels, status widgets, release publication or later application work belongs
to this delta. R1 has not begun in R0.
