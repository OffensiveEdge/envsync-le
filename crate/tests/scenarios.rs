//! The tier that needs a tree larger than an editor opens.
//!
//! Gated behind `ENVSYNC_LE_SCENARIOS` and run by CI. Nothing here
//! substitutes for the unit tests, which run everywhere on every push.
//!
//! **A skipped scenario is never reported as a pass.**

use std::fmt::Write as _;

fn enabled(name: &str) -> bool {
    if std::env::var_os("ENVSYNC_LE_SCENARIOS").is_some() {
        return true;
    }
    eprintln!("SKIPPED {name}: set ENVSYNC_LE_SCENARIOS to run it");
    false
}

struct Tree(std::path::PathBuf);

impl Tree {
    fn new(name: &str) -> Self {
        let root = std::env::temp_dir().join(format!("envsync-le-scenario-{name}"));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("a temporary directory");
        Self(root)
    }
    fn write(&self, relative: &str, contents: &str) {
        let target = self.0.join(relative);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).expect("a parent directory");
        }
        std::fs::write(target, contents).expect("a file");
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn run(args: &[&str]) -> (i32, serde_json::Value) {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_envsync-le"))
        .args(args)
        .output()
        .expect("the binary runs");
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    (
        output.status.code().expect("an exit code"),
        serde_json::from_str(stdout.trim()).expect("stdout carries JSON"),
    )
}

/// Comparison is quadratic if done naively: every file against every
/// reference key, and every mismatch searching for a file that has the
/// keys. A monorepo with a service per directory is the real shape.
#[test]
fn a_monorepo_of_services_completes() {
    if !enabled("a_monorepo_of_services_completes") {
        return;
    }
    let tree = Tree::new("monorepo");
    let mut shared = String::new();
    for key in 0..200 {
        let _ = writeln!(shared, "SHARED_KEY_{key}=x");
    }
    for service in 0..300 {
        // Every tenth service is missing one key, so the answer is not
        // trivially "all the same".
        let content = if service % 10 == 0 {
            shared.replace("SHARED_KEY_7=x\n", "")
        } else {
            shared.clone()
        };
        tree.write(&format!("services/s{service}/.env"), &content);
    }

    let (code, report) = run(&[&tree.0.to_string_lossy()]);
    assert_eq!(code, 1);
    assert_eq!(report["summary"]["files"], 300);
    assert_eq!(report["summary"]["missing"], 30);
}

/// One file with a great many keys, which is what a `.env.example` in a
/// mature service looks like.
#[test]
fn a_file_with_many_keys_completes() {
    if !enabled("a_file_with_many_keys_completes") {
        return;
    }
    let tree = Tree::new("wide");
    let mut template = String::new();
    for key in 0..20_000 {
        let _ = writeln!(template, "KEY_{key}=");
    }
    tree.write(".env.example", &template);
    tree.write(".env", "KEY_0=set\n");

    let (code, report) = run(&["--template", ".env.example", &tree.0.to_string_lossy()]);
    assert_eq!(code, 1);
    assert_eq!(
        report["missingKeys"][0]["keys"]
            .as_array()
            .expect("a list")
            .len(),
        19_999
    );
}

/// A file whose quote never closes swallows the rest of it, so a large
/// one becomes a large single value. The parser must not go quadratic
/// scanning for a close that is not there.
#[test]
fn a_large_unterminated_value_completes() {
    if !enabled("a_large_unterminated_value_completes") {
        return;
    }
    let tree = Tree::new("unterminated");
    let mut content = String::from("BEFORE=1\nBROKEN=\"never closed\n");
    for line in 0..100_000 {
        let _ = writeln!(content, "LOST_{line}=x");
    }
    tree.write(".env", &content);
    tree.write(".env.production", "BEFORE=1\nBROKEN=2\n");

    let (code, report) = run(&[&tree.0.to_string_lossy()]);
    assert_eq!(
        code, 0,
        "both files have BEFORE and BROKEN and nothing else"
    );
    assert_eq!(report["files"][0]["keys"], 2);
}
