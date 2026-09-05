# REPLIA

REPLIA is an embeddable terminal interaction library for building robust
line-oriented and REPL-style command interfaces.

**Early / pre-release:** this repository currently contains a compilable Rust
library foundation, architectural boundaries, and package integrity checks.
There is no usable line editor or public operational API yet. The crate is not
published, and Cargo publication is disabled.

The intended scope is input editing, terminal state restoration, history
navigation, host-provided completion, and coordination between input and
external output. These are design targets, not implemented features. Hosts
retain their command language, application state, prompt content, and the
meaning of submitted input and interrupts.

The library is intended for interpreters, database and debugger shells,
developer tools, and other interactive command-line applications. It does not
aim to provide a dashboard or application framework.

## Development

Use stable Rust with the `rustfmt` and `clippy` components. From this directory:

```sh
cargo fmt --check
cargo check --all-targets
cargo test --all-targets
cargo clippy --all-targets --all-features -- -D warnings
```

The foundation has no third-party dependencies. CI checks Linux; other
platforms and a minimum supported Rust version have not been qualified.

See [architecture](docs/architecture.md) for ownership and the next bounded
implementation step, and [contributing](CONTRIBUTING.md) for validation policy.

## License

[MIT](LICENSE).
