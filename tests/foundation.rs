//! Package checks for the initial library foundation, not editor behavior.

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
fn foundation_dependency_graph_contains_only_the_library() {
    let graph = cargo_output(&[
        "tree",
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
    assert_eq!(nodes.len(), 1, "R0 must have no dependencies: {graph}");
    let identity = format!("{} v{} ", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    assert!(nodes[0].starts_with(&identity), "unexpected root: {graph}");
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
        "LICENSE",
        "README.md",
        "CONTRIBUTING.md",
        "docs/architecture.md",
        "tests/foundation.rs",
    ] {
        assert!(
            inventory.lines().any(|path| path == required),
            "package is missing {required}: {inventory}"
        );
    }
    assert!(!inventory.lines().any(|path| path.starts_with("target/")));
}
