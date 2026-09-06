<p align="center">
  <img src="assets/replai-logo.png" alt="REPLAI logo" width="320">
</p>

# REPLAI

REPLAI is an embeddable terminal interaction library for building robust
line-oriented and REPL-style command interfaces.

**Early / pre-release:** a working Rust editor and Linux TTY backend are now
available, with a separately qualified C binding (pre-release ABI 1). Both
boundaries remain experimental; no stable ABI or crate release is promised.
Cargo publication remains disabled.

The editor supports grapheme movement and deletion, bounded UTF-8 drafts,
in-memory history with draft return, bracketed multiline paste, host completion,
interrupt/EOF delivery, resize and coordinated external output. Presentation
stays line-oriented: an accented prompt, compact continuation lines and the
terminal's default background. `NO_COLOR` and `TERM=dumb` disable text styling.

Hosts own the application loop, prompt content, command language, history
admission, completion candidates and the meaning of input and interrupts.
REPLAI is suitable for experimenting with interpreters, database and debugger
shells, developer tools and ordinary interactive command interfaces. It does
not supply application commands, a scheduler or a dashboard.

## Where REPLAI fits in a REPL

A **Read–Eval–Print Loop** repeatedly reads an expression, evaluates it in an
environment, and displays its result. The classic teaching reference is
[SICP §4.1.4](https://sicp.sourceacademy.org/chapters/4.1.4.html).
REPLAI supplies the terminal interaction part; the host supplies the language,
evaluation, result formatting and loop policy.

```mermaid
flowchart LR
    edit["REPLAI<br/>Prompt and edit UTF-8"] --> read["Host<br/>Read / parse"]
    read --> evaluate["Host<br/>Evaluate / execute"]
    evaluate --> print["Host<br/>Format and print"]
    print -->|Next interaction| edit
```

Editing text and reading a language expression are different operations. This
library/application separation also appears in
[GNU Readline's programming interface](https://web.mit.edu/gnu/doc/html/rlman_2.html).
See [the classical REPL and our design](docs/repl.md) for the complete teaching
diagram, a worked example, and references including McCarthy's 1960 paper.

## Try the reference fixture

On Linux, in an interactive ANSI/VT-compatible terminal:

```sh
cargo run --example demo
# Optional: a notice arrives after two seconds, preserving an unfinished draft.
cargo run --example demo -- --notice
```

```text
demo> hello
echo: hello

demo>
```

Tab completes a unique match from `hello`, `help`, `world`. Up/Down recall
host-admitted input; Home/End and Ctrl-A/Ctrl-E move across the whole draft.
Ctrl-C returns an interrupt; this fixture clears its draft and starts again.
Ctrl-D exits with an empty draft, otherwise deletes the next grapheme. Pasted
newlines remain one input until Enter. Ctrl-L explicitly clears and redraws.

The host API is illustrated in [the fixture](examples/demo.rs) and crate docs
(`cargo doc --no-deps`). Applications own an `Interaction` containing their `Editor`, explicitly open
and close its terminal resource, poll typed `Event`s and call `complete` or
`external_output`. They decide what happens after the terminal is restored.

## C consumers

Build static/shared artifacts and stage the self-contained header and pkg-config
metadata into an empty prefix:

```sh
cargo build --locked --release -p replai-c
python3 tools/stage_c.py --prefix /tmp/replai-install
```

The [generic C demo](examples/c/demo.c) uses only the installed `replai.h` and
library. See [C API and installation](docs/c-api.md) for compiler commands,
ABI 1 ownership, events, failure rules and the complete qualification path.

## Development and qualification

Use current stable Rust with `rustfmt` and `clippy`:

```sh
cargo fmt --check
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --doc
cargo test --workspace --release
git diff --check
```

Tests include deterministic editing/protocol cases and real Linux PTYs with a
VT terminal-state oracle. `python3 tools/qualify.py` additionally qualifies
installed static/shared C consumers, ABI layout and memory ownership with
Valgrind and Linux Landlock isolation. No sibling repository or application runtime is
needed. Other operating systems, every terminal's Unicode/font/reflow behavior,
a minimum Rust version and production maturity have not been qualified.

See [architecture](docs/architecture.md) for ownership, Unicode and lifecycle
contracts, [presentation evidence](docs/presentation.md) for the reference
comparison, and [contributing](CONTRIBUTING.md) for development policy.

## License

[MIT](LICENSE).
