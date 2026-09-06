//! Package integrity and dependency-source independence.

use std::process::Command;

fn cargo_output(arguments: &[&str]) -> String {
    let output = Command::new(env!("CARGO"))
        .args(arguments)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .env("CARGO_TERM_COLOR", "never")
        .output()
        .expect("Cargo must be available to verify the package");
    assert!(
        output.status.success(),
        "cargo {arguments:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("Cargo output must be UTF-8")
}

#[test]
fn dependency_graph_has_one_local_root_and_registry_only_dependencies() {
    let graph = cargo_output(&[
        "tree",
        "--package",
        "replai",
        "--locked",
        "--offline",
        "--all-features",
        "--target",
        "all",
        "--edges",
        "normal,build,dev",
        "--prefix",
        "none",
        "--format",
        "{p}",
    ]);
    let nodes: Vec<_> = graph.lines().collect();
    assert!(!nodes.is_empty());
    let identity = format!("{} v{} ", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    assert!(nodes[0].starts_with(&identity), "unexpected root: {graph}");
    for node in &nodes[1..] {
        // Registry nodes have only a name/version (and optional dedupe marker).
        // Local or Git dependencies introduce another parenthesized source.
        let node = node.strip_suffix(" (*)").unwrap_or(node);
        assert!(!node.contains('('), "non-registry dependency: {node}");
    }
}

#[test]
fn distributable_contains_source_license_and_development_contract() {
    // Listing is read-only and does not publish or build a release.
    let inventory = cargo_output(&[
        "package",
        "--list",
        "--locked",
        "--offline",
        "--allow-dirty",
    ]);
    for required in [
        "Cargo.toml",
        "Cargo.lock",
        "src/lib.rs",
        "src/core.rs",
        "src/input.rs",
        "src/presentation.rs",
        "src/terminal.rs",
        "examples/demo.rs",
        "LICENSE",
        "README.md",
        "CONTRIBUTING.md",
        "AGENTS.md",
        "ROADMAP.md",
        "CHANGELOG.md",
        "docs/README.md",
        "docs/repl.md",
        "docs/architecture.md",
        "docs/interaction.md",
        "docs/presentation.md",
        "docs/c-api.md",
        "docs/development.md",
        "tests/fixtures/presentation.tsv",
        "tests/pty.rs",
        "tests/foundation.rs",
    ] {
        assert!(
            inventory.lines().any(|path| path == required),
            "package is missing {required}: {inventory}"
        );
    }
    assert!(!inventory.lines().any(|path| path.starts_with("target/")));
}
