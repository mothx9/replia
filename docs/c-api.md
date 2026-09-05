# C boundary — pre-release ABI 1

REPLIA exposes one Linux-qualified C ABI through the separate `replia-c`
package. ABI 1 identifies an exact header/binary contract; it is not a promise
of long-term ABI or SemVer stability. No release has been published.

## Build, install and link

Producer requirements: current stable Rust, Python 3 and Linux. Consumer
requirements: C11 compiler and standard headers; pkg-config is convenient.
The C consumer does not need Cargo, Rust sources or private headers.

```sh
cargo build --locked --release -p replia-c
python3 tools/stage_c.py --prefix /tmp/replia-install
```

The prefix must be absent or empty. Installation writes only that prefix:

```text
include/replia.h
lib/libreplia_c.a
lib/libreplia_c.so
lib/pkgconfig/replia.pc
share/licenses/replia/LICENSE
```

Copy `examples/c/demo.c` into an unrelated build directory, then:

```sh
export PKG_CONFIG_PATH=/tmp/replia-install/lib/pkgconfig
cc -std=c11 -Wall -Wextra -Wpedantic -Werror demo.c \
  $(pkg-config --cflags --libs replia) \
  -Wl,-rpath,/tmp/replia-install/lib -o demo
./demo
```

For static REPLIA linkage use the archive explicitly (system dependencies may
remain shared):

```sh
cc -std=c11 -Wall -Wextra -Wpedantic -Werror demo.c \
  $(pkg-config --cflags replia) /tmp/replia-install/lib/libreplia_c.a \
  $(pkg-config --libs-only-other --libs-only-l --static replia | sed 's/-lreplia_c//g') \
  -o demo-static
```

The `.pc` file derives paths from its installed location. It contains no author
workstation paths. Shared consumers must arrange loader search explicitly;
qualification verifies `ldd` resolves the staged `.so`. Mixing another REPLIA
copy into the same process is unsupported: the active-terminal lease is per
linked library image, not an inter-library or inter-process lock.

## Ownership and lifecycle

Initialize `replia_config` to zero, set `struct_size = sizeof(config)`,
`abi_version = REPLIA_C_ABI_VERSION`, the byte capacity and history entry bound.
Compare `replia_abi_version(&version)` with the header macro before creation.
Pass an initialized NULL handle slot to `replia_create`. The library owns the
resulting allocation. `replia_destroy(&handle)` is its only release operation;
it closes any active terminal and sets the slot to NULL, including when close
reports an I/O failure. A second destroy of NULL returns INVALID_ARGUMENT.

The normal sequence is create → optional prompt/draft/history configuration →
open → poll/complete/output → outcome/close → configure/reopen or destroy.
Prompt and direct draft/history mutation require a closed interaction. `close`
is idempotent for a valid handle. Submit, interrupt and EOF restore the terminal
and release its duplicated descriptors. They retain the editor and history;
clearing and history admission remain host decisions. Successful reopen clears
the previous submitted-text snapshot, but failed open preserves it.

Input/output FDs are borrowed only during `open`. The library duplicates what
it needs; callers retain ownership and may close their originals after a
successful open. During the call they must keep the descriptors valid. The
library closes only its duplicates. Both FDs must designate the same suitable
TTY. Opening while already active returns INVALID_STATE. A second interaction
in the same library instance returns BUSY, even on another TTY.

Calls using the same handle must be serialized by the caller, including reads
and destruction. No concurrent close/destroy is permitted. Coordinate all other
terminal writers, mode changes and environment changes while opening. This is
not an internally synchronized or broadly thread-safe API.

## Records, pointers and UTF-8

All externally governed tags are fixed-width integers, not C enums. Config and
event records have `uint32_t struct_size`, `uint32_t abi_version`, explicit
fields and two reserved `uint64_t`s. Size must equal the ABI 1 size; version must
be 1; reserved fields must be zero. Before **each** poll/interrupt initialize a
fresh zero event and set its size/version. Output fields are not request flags.
Unsupported size/version is ABI_MISMATCH, not a request to guess a layout.

Every text argument is pointer plus byte length. NULL is permitted only with
zero length. Storage must be live, readable for that extent and valid UTF-8.
No implicit `strlen` occurs. Length/address overflow, detectable alignment
errors and NULL misuse return INVALID_ARGUMENT. Arbitrary non-NULL invalid,
dangling, forged or inaccessible pointers cannot be reliably detected; those
remain caller contract violations. Record pointers must provide at least their
aligned size prefix and the full extent advertised by that prefix. Output
storage must be writable, properly aligned and disjoint from other arguments.
No reference, slice, Rust enum, bool, String or allocator-owned text crosses C.

Draft text accepts LF and TAB; other controls, including NUL and ESC, fail.
Prompt fields accept no controls and are bounded to 1024 bytes each. They are
literal label, suffix and continuation text, without ANSI. External text accepts
LF/TAB and normalizes CRLF; rejected controls cannot inject terminal escapes.
Core capacity is measured in UTF-8 bytes. C cursor/range values are byte offsets
on extended grapheme boundaries, not scalar indices or display cells.

`draft_copy` and `submitted_copy` use caller-owned buffers. NULL/zero capacity
queries the exact required byte count. Exact-size and larger buffers copy that
many bytes, **without a NUL terminator**. A too-small buffer returns
BUFFER_TOO_SMALL, writes the required count (and draft cursor), and leaves all
buffer bytes untouched. It never produces a partial successful string.
`submitted_copy` requires an earlier submission and returns that entire input,
including newlines. No separate text-free function exists or is needed.

## Events and errors

All functions return `replia_status` (`int32_t`). Event outputs are meaningful
only on OK. On a failed poll/interrupt the caller's event record stays unchanged.

| Event kind | Value | Host action / terminal state |
| --- | ---: | --- |
| NONE | 0 | No event this poll; continue; active |
| SUBMITTED | 1 | Copy complete input, decide admission/execution; closed |
| INTERRUPTED | 2 | Decide cancellation/clear/retry; closed |
| END_OF_INPUT | 3 | Decide exit; closed |
| COMPLETION_REQUESTED | 4 | Read draft/cursor and choose replacement; active |
| EDIT_REJECTED | 5 | `event.status` gives edit rejection; draft unchanged; active |

`text_bytes` and `cursor_bytes` describe the current draft at the event. On
submission `text_bytes` also matches the retained submitted snapshot. A call
returning OK can still report EDIT_REJECTED; it is not a terminal I/O error.
Each poll waits at most 100 ms, even with a larger requested timeout, and consumes
at most one input byte. Host scheduling controls latency and output timing.

| Status | Value | Contract |
| --- | ---: | --- |
| OK | 0 | Operation completed |
| INVALID_ARGUMENT | 1 | Detectable pointer/length/tag/reserved/FD misuse |
| INVALID_UTF8 | 2 | Invalid UTF-8 span |
| INVALID_RANGE | 3 | Unordered/out-of-bounds/non-grapheme replacement range |
| CAPACITY | 4 | Configured text or prompt bound exceeded |
| INVALID_STATE | 5 | Operation unavailable in current lifecycle |
| UNSUITABLE_TERMINAL | 6 | Non-TTY, mismatched TTY or unsupported dimensions |
| IO | 7 | Terminal syscall/read/write/cleanup failure |
| BUFFER_TOO_SMALL | 8 | Query reported required bytes; destination unchanged |
| ABI_MISMATCH | 9 | Unsupported record size/version |
| BUSY | 10 | Another active interaction owns the library lease |
| INTERNAL | 11 | A Rust panic was contained |
| INVALID_TEXT | 12 | Unsupported control text |
| HISTORY_DISABLED | 13 | Host configured no history entries |
| INVALID_SEQUENCE | 14 | Rejected terminal input sequence |

`replia_status_text` copies a generic diagnostic using the same buffer contract;
unknown codes return INVALID_ARGUMENT. Human messages are not machine tags. ABI 1
does not retain per-handle errno or OS-specific diagnostic strings.

Rejected validation operations preserve draft, cursor, history and terminal
state. Invalid event/config records do not consume input. Capacity and completion
failures are atomic. NULL/zero `set_draft` is valid and intentionally clears the
draft. Buffer-too-small may update required/cursor output fields as specified.
I/O failure is different: visible output may be partial; restoration is attempted
and the host should close/destroy before deciding whether reopening is safe.

## Completion, history and external output

Tab yields COMPLETION_REQUESTED. Read draft and byte cursor, discover candidates
outside the library, then call `replia_complete(start, end, bytes, length)` for a
host-selected replacement. No callback or registry exists. No candidates, multiple
candidates without a choice, or host lookup failure require no mutation.
Invalid UTF-8, capacity and grapheme-splitting ranges fail before redraw.

`replia_history_add` admits one entry while closed; the host owns persistence,
privacy, deduplication and admission. Up/Down use the same Rust history mechanism,
including restoring the pre-history draft and cursor. History zero is allowed.

`replia_external_output(role, bytes, length)` delegates to the single Rust
renderer. Roles are DEFAULT=0, STRONG=1, ACCENT=2, DIM=3, SUCCESS=4, WARNING=5,
ERROR=6. The transaction validates text, suspends the visible editing surface,
writes a complete host text line, then redraws exact draft/cursor. It is a
synchronous line transaction, not an arbitrary ANSI writer or no-newline stream.
The terminal background remains default; no alternate screen is introduced.

## Restoration, signals and panic containment

The native backend restores the exact captured termios on normal outcomes,
close, read/write failure, partial acquisition and ordinary unwinding. Cleanup
also disables bracketed paste and releases owned FDs. A disconnected terminal
can refuse restoration; IO reports that limit. Drop is best effort and does not
panic or call process exit. No library can clean up after SIGKILL or process abort.

No signal handler, mask, background thread or cancellation policy is installed.
Raw-mode Ctrl-C is a decoded byte; explicit `replia_interrupt` supports a host
signal policy. Resize uses dimension polling. Signal handlers must not call
REPLIA: these functions are not async-signal-safe. Notify the serialized host loop.

Every exported function uses the same `catch_unwind` guard. The binding refuses
panic=abort builds. A contained operational panic returns INTERNAL, poisons the
handle and attempts close. Only close/destroy are then supported. Panic hooks
remain the host's Rust process policy. A panic payload whose own destructor
panics is separately caught; its secondary payload is deliberately retained to
prevent another unwind across C. This exceptional case is not a zero-allocation
recovery promise. Normal and tested error paths free every allocation. Allocation
failure or foreign undefined behavior cannot be converted into safe Rust unwind.

## Executable authority and current limits

`api/c-abi.json` owns the narrow ABI declarations. `tools/generate_abi.py --check`
checks generated C/Rust records, numeric constants, probes and Rust function
pointer signature assertions. The real C and Rust probes compare size, alignment
and every field offset/tag. The C binding imports only public `replia` items.
Only its `src/lib.rs` dereferences C pointers, constructs borrowed FDs or manages
the opaque allocation. The implementation crate still forbids unsafe code.

Run the complete qualification with current stable Rust, cc/c++, pkg-config,
Python 3, Valgrind and a Linux kernel with Landlock:

```sh
python3 tools/qualify.py
```

The command builds/tests Rust, stages release artifacts, compiles the independent
C consumer and layout/header probes, executes static/shared PTYs and adversarial
contracts, checks FD ownership and Valgrind, audits symbols/loader/metadata, runs
release Rust regressions, then requires a clean repository. `--allow-dirty` is a
development aid and explicitly does not qualify the clean closure gate.
`--work /tmp/new-directory` retains commands, layout, misuse, PTY JSON and memory
reports. The native CI separates prepare/static/shared/memory/audit gates.
Consumer compilation and execution run with Landlock denying repository reads;
only staged artifacts, copied C files and system toolchain/runtime are available.

The terminal backend is Linux-qualified. C11 and C++17 inclusion are tested;
that does not establish another OS backend or every compiler/architecture ABI.
Unicode width, terminal reflow, TERM=dumb, unframed paste and external-output
limitations remain those in [architecture](architecture.md). No consumer
integration or release is performed by this qualification.
