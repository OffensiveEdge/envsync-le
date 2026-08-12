//! Does the crate open what it claims?
//!
//! The extractor crates in this family answer that with one file per
//! extension in their alias table. This one has no format table: what it
//! claims is a **file-name classifier** — which names are dotenv files,
//! and what kind each is — so that is what the matrix covers.
//!
//! Every classification case in the shared corpus is written into a real
//! tree and run through the real binary. A name that classifies
//! correctly in a unit test but that the walk never reaches is a crate
//! that opens fewer files than it says it does, and nothing else here
//! would notice.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

const BINARY: &str = env!("CARGO_BIN_EXE_envsync-le");
/// The contract between this crate and the extension. Read here rather
/// than restated, so a name added there is a name checked here.
const CORPUS: &str = include_str!("../fixtures/detection.json");

/// The documented types. A seventh appearing in the classifier with no
/// case in the corpus is a hole this test exists to make visible.
const DOCUMENTED_TYPES: [&str; 6] = [
    "base",
    "local",
    "example",
    "production",
    "development",
    "test",
];

/// Names the crate must **not** claim, including the ones that look as
/// though it should. `.ENV` is the sharp one: classification is
/// case-sensitive on purpose, because the extension compares exactly
/// this way and matching loosely would have the two disagree about which
/// files exist.
const NOT_DOTENV: [&str; 12] = [
    ".env-backup",
    ".env_local",
    ".envrc",
    "env",
    "envrc",
    "dotenv",
    "env.example",
    "environment",
    "notenv",
    "README.md",
    "package.json",
    "config.yaml",
];

static COUNTER: AtomicUsize = AtomicUsize::new(0);

struct Tree {
    root: PathBuf,
}

impl Tree {
    fn new(name: &str) -> Self {
        let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "envsync-le-matrix-{name}-{}-{unique}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("a temporary directory");
        Self {
            root: std::fs::canonicalize(&root).expect("a canonical directory"),
        }
    }

    fn path(&self) -> &Path {
        &self.root
    }

    fn write(&self, relative: &str, contents: &str) {
        let target = self.root.join(relative);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).expect("a parent directory");
        }
        std::fs::write(&target, contents).expect("a file");
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn report(root: &Path) -> serde_json::Value {
    let output = Command::new(BINARY)
        .arg(root)
        .stdin(Stdio::null())
        .output()
        .expect("the binary runs");
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    serde_json::from_str(stdout.trim()).expect("stdout carries one JSON report")
}

fn corpus() -> Vec<serde_json::Value> {
    let parsed: serde_json::Value = serde_json::from_str(CORPUS).expect("the corpus is JSON");
    parsed["classification"]
        .as_array()
        .expect("classification cases")
        .clone()
}

/// **The matrix.** Every name the corpus classifies, written to disk and
/// run through the binary: the ones it claims must come back with the
/// documented type, and the ones it does not claim must not come back at
/// all.
#[test]
fn every_classified_name_is_opened_and_typed_as_documented() {
    let cases = corpus();
    assert!(
        cases.len() >= 12,
        "the corpus classifies too little to be a matrix"
    );

    let tree = Tree::new("classifier");
    for case in &cases {
        let path = case["path"].as_str().expect("a path");
        tree.write(path, "SHARED=1\n");
    }
    for name in NOT_DOTENV {
        tree.write(name, "SHARED=1\n");
    }

    let report = report(tree.path());
    let files = report["files"].as_array().expect("files");
    let seen: std::collections::HashMap<&str, &str> = files
        .iter()
        .map(|file| {
            (
                file["path"].as_str().expect("a path"),
                file["type"].as_str().expect("a type"),
            )
        })
        .collect();

    let mut missing = Vec::new();
    let mut mistyped = Vec::new();
    let mut claimed = Vec::new();
    for case in &cases {
        let path = case["path"].as_str().expect("a path");
        let expected = case["type"].as_str().expect("a type");
        match (case["isEnv"].as_bool().expect("isEnv"), seen.get(path)) {
            (true, None) => missing.push(path.to_string()),
            (true, Some(actual)) if *actual != expected => {
                mistyped.push(format!("{path}: {actual}, documented as {expected}"));
            }
            (false, Some(actual)) => claimed.push(format!("{path}: opened as {actual}")),
            _ => {}
        }
    }
    for name in NOT_DOTENV {
        if let Some(actual) = seen.get(name) {
            claimed.push(format!("{name}: opened as {actual}"));
        }
    }

    assert!(
        missing.is_empty(),
        "the walk never opened these documented dotenv files: {missing:?}\n{report}"
    );
    assert!(
        mistyped.is_empty(),
        "classified against the corpus: {mistyped:?}"
    );
    assert!(
        claimed.is_empty(),
        "these are not dotenv files and were opened anyway: {claimed:?}"
    );
}

/// A type the classifier can produce and the corpus never exercises is a
/// hole. Held both ways, so neither list can grow past the other in
/// silence.
#[test]
fn every_documented_type_has_a_case_and_no_case_has_an_undocumented_type() {
    let cases = corpus();
    let in_corpus: std::collections::BTreeSet<String> = cases
        .iter()
        .filter(|case| case["isEnv"].as_bool().expect("isEnv"))
        .map(|case| case["type"].as_str().expect("a type").to_string())
        .collect();
    let documented: std::collections::BTreeSet<String> = DOCUMENTED_TYPES
        .iter()
        .map(|name| (*name).to_string())
        .collect();
    assert_eq!(
        in_corpus, documented,
        "the corpus and the documented types have drifted apart"
    );
}

/// Case sensitivity is the one the extension pins: `.ENV` is not a
/// dotenv file, and inventing one to be helpful would have the two
/// frontends disagree about which files exist. Each variant gets its own
/// directory — on a case-folding filesystem they are otherwise one file.
#[test]
fn upper_case_variants_are_not_dotenv_files() {
    let tree = Tree::new("casing");
    tree.write("real/.env", "SHARED=1\n");
    for (index, name) in [".ENV", ".Env", ".ENV.production"].iter().enumerate() {
        tree.write(&format!("variant{index}/{name}"), "SHARED=1\n");
    }

    let report = report(tree.path());
    let files = report["files"].as_array().expect("files");
    assert_eq!(
        files.len(),
        1,
        "an upper-case variant was opened as a dotenv file: {report}"
    );
    assert_eq!(files[0]["path"], "real/.env");
}
