//! The exit codes and the stdout contract, driven against the built
//! binary.
//!
//! These are the API — and here more than anywhere else in the family,
//! because this crate's whole product is the exit code. A shell branches
//! on it, so it is pinned by tests that drive the real binary against a
//! real tree.
//!
//! A new refusal adds its case here.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

const BINARY: &str = env!("CARGO_BIN_EXE_envsync-le");
static COUNTER: AtomicUsize = AtomicUsize::new(0);

struct Tree {
    root: PathBuf,
}

impl Tree {
    fn new(name: &str) -> Self {
        let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "envsync-le-contract-{name}-{}-{unique}",
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

    fn write(&self, relative: &str, contents: &str) -> PathBuf {
        let target = self.root.join(relative);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).expect("a parent directory");
        }
        std::fs::write(&target, contents).expect("a file");
        target
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

struct Run {
    code: i32,
    stdout: String,
    stderr: String,
}

fn run(args: &[&str]) -> Run {
    let output = Command::new(BINARY)
        .args(args)
        .output()
        .expect("the binary runs");
    Run {
        code: output.status.code().expect("an exit code"),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

/// The whole report, parsed. Doubles as the assertion that stdout is one
/// JSON object and nothing else.
fn report(run: &Run) -> serde_json::Value {
    serde_json::from_str(run.stdout.trim()).expect("stdout carries one JSON report")
}

fn drifted(name: &str) -> Tree {
    let tree = Tree::new(name);
    tree.write(".env.example", "DATABASE_URL=\nREDIS_URL=\nSENTRY_DSN=\n");
    tree.write(
        ".env",
        "DATABASE_URL=postgres://dev\nREDIS_URL=redis://dev\nLOCAL_ONLY=1\n",
    );
    tree.write("README.md", "not a dotenv file\n");
    tree
}

#[test]
fn files_in_sync_exit_zero() {
    let tree = Tree::new("sync");
    tree.write(".env", "A=1\nB=2\n");
    tree.write(".env.production", "A=3\nB=4\n");
    let run = run(&[&tree.path().to_string_lossy()]);
    assert_eq!(run.code, 0, "{}", run.stderr);
    assert_eq!(report(&run)["status"], "in-sync");
    assert!(run.stderr.contains("in sync"), "{}", run.stderr);
}

#[test]
fn a_missing_key_exits_one() {
    let tree = drifted("missing");
    let run = run(&[&tree.path().to_string_lossy()]);
    assert_eq!(run.code, 1, "{}", run.stderr);
    assert_eq!(report(&run)["status"], "missing-keys");
    assert!(run.stderr.contains("SENTRY_DSN"), "{}", run.stderr);
}

/// A repository with no dotenv files is not out of sync. Failing a build
/// over it would be the tool inventing a problem.
#[test]
fn no_dotenv_files_exits_zero() {
    let tree = Tree::new("none");
    tree.write("README.md", "nothing here\n");
    let run = run(&[&tree.path().to_string_lossy()]);
    assert_eq!(run.code, 0);
    assert_eq!(report(&run)["status"], "no-files");
    assert!(run.stderr.contains("no dotenv files"), "{}", run.stderr);
}

/// The difference between the two modes, end to end. Without a template
/// nothing can be extra; with one, the same tree fails.
#[test]
fn a_template_turns_a_local_key_into_a_finding() {
    let tree = Tree::new("template");
    tree.write(".env.example", "A=\n");
    tree.write(".env", "A=1\nLOCAL_ONLY=2\n");

    let auto = run(&[&tree.path().to_string_lossy()]);
    assert_eq!(auto.code, 1, "LOCAL_ONLY is missing from the example");
    assert_eq!(
        report(&auto)["extraKeys"].as_array().expect("a list").len(),
        0
    );

    let templated = run(&["--template", ".env.example", &tree.path().to_string_lossy()]);
    assert_eq!(templated.code, 1);
    let report = report(&templated);
    assert_eq!(report["mode"], "template");
    assert_eq!(report["extraKeys"][0]["keys"][0], "LOCAL_ONLY");
    assert!(templated.stderr.contains("extra"), "{}", templated.stderr);
}

/// **The property this tool exists under.** A dotenv file is where
/// credentials live and no value may ever reach the output.
#[test]
fn no_value_ever_reaches_stdout_or_stderr() {
    let tree = Tree::new("values");
    tree.write(".env.example", "SECRET=\nDATABASE_URL=\n");
    tree.write(
        ".env",
        "SECRET=hunter2\nDATABASE_URL=postgres://user:pw@host/db\nEXTRA=x\n",
    );
    let run = run(&["--template", ".env.example", &tree.path().to_string_lossy()]);
    let everything = format!("{}{}", run.stdout, run.stderr);
    assert!(!everything.contains("hunter2"), "{everything}");
    assert!(!everything.contains("postgres"), "{everything}");
    assert!(!everything.contains("user:pw"), "{everything}");
    assert!(everything.contains("EXTRA"), "the key name is the finding");
}

#[test]
fn ignoring_case_can_bring_a_tree_into_sync() {
    let tree = Tree::new("case");
    tree.write(".env", "database_url=1\n");
    tree.write(".env.production", "DATABASE_URL=2\n");
    assert_eq!(run(&[&tree.path().to_string_lossy()]).code, 1);
    assert_eq!(
        run(&["--ignore-case", &tree.path().to_string_lossy()]).code,
        0
    );
}

#[test]
fn an_exclude_pattern_removes_a_file_from_the_comparison() {
    let tree = Tree::new("exclude");
    tree.write(".env", "A=1\n");
    tree.write(".env.production.local", "A=1\nEXTRA=2\n");
    assert_eq!(run(&[&tree.path().to_string_lossy()]).code, 1);
    let excluded = run(&["--exclude", ".env.*.local", &tree.path().to_string_lossy()]);
    assert_eq!(excluded.code, 0, "{}", excluded.stderr);
}

/// A dotenv file is hidden by definition, so `--hidden` is about
/// directories. Getting it backwards would make the default find
/// nothing at all.
#[test]
fn hidden_controls_directories_not_the_dotenv_files() {
    let tree = Tree::new("hidden");
    tree.write(".env", "A=1\n");
    tree.write(".cache/.env", "A=1\nEXTRA=2\n");
    assert_eq!(
        run(&[&tree.path().to_string_lossy()]).code,
        0,
        "the hidden directory is skipped, and .env itself is not"
    );
    assert_eq!(run(&["--hidden", &tree.path().to_string_lossy()]).code, 1);
}

#[test]
fn named_files_replace_discovery() {
    let tree = drifted("named");
    let run = run(&[
        "--file",
        &tree.path().join(".env").to_string_lossy(),
        "--file",
        &tree.path().join(".env.example").to_string_lossy(),
    ]);
    assert_eq!(report(&run)["summary"]["files"], 2);
}

#[test]
fn an_unreadable_input_exits_two() {
    assert_eq!(run(&["/no/such/place-xyz"]).code, 2);
}

/// A file that does not parse cleanly is a warning naming it, not a
/// failed run — the keys that did parse are still evidence.
#[test]
fn a_parse_error_warns_without_becoming_an_exit_two() {
    let tree = Tree::new("broken");
    tree.write(".env", "GOOD=1\n2BAD=2\n");
    tree.write(".env.production", "GOOD=1\n");
    let run = run(&[&tree.path().to_string_lossy()]);
    assert_eq!(run.code, 0, "{}", run.stderr);
    assert!(run.stderr.contains("Invalid key format"), "{}", run.stderr);
}

#[test]
fn an_unknown_flag_exits_two_and_names_itself() {
    let tree = drifted("badflag");
    let run = run(&["--tempate", &tree.path().to_string_lossy()]);
    assert_eq!(run.code, 2);
    assert!(run.stderr.contains("--tempate"), "{}", run.stderr);
    assert!(run.stdout.is_empty(), "a refusal writes no report");
}

/// The tool reads names, never values, and no flag may suggest
/// otherwise.
#[test]
fn no_flag_offers_to_read_a_value() {
    let tree = drifted("novalues");
    for attempt in [
        "--values",
        "--show-values",
        "--compare-values",
        "--fix",
        "--write",
    ] {
        assert_eq!(
            run(&[attempt, &tree.path().to_string_lossy()]).code,
            2,
            "{attempt} was accepted"
        );
    }
}

#[test]
fn version_and_help_exit_clear() {
    let version = run(&["--version"]);
    assert_eq!(version.code, 0);
    assert!(version.stdout.contains("envsync-le"));
    let help = run(&["--help"]);
    assert_eq!(help.code, 0);
    assert!(help.stdout.contains("usage: envsync-le"));
    assert!(help.stdout.contains("union"), "auto mode is unexplained");
}

#[test]
fn stdout_carries_one_report_and_stderr_only_the_summary() {
    let tree = drifted("streams");
    let run = run(&[&tree.path().to_string_lossy()]);
    assert_eq!(
        run.stdout.trim().lines().count(),
        1,
        "one report, not one per file"
    );
    assert!(!run.stderr.contains('{'), "{}", run.stderr);
    assert!(run.stderr.contains("out of sync"), "{}", run.stderr);
}

/// **The cross-surface contract.** Both surfaces call one entry point,
/// so they must answer identically for the same tree.
#[test]
fn the_cli_and_the_mcp_server_report_the_same_thing() {
    let tree = drifted("agreement");
    let from_cli = report(&run(&[&tree.path().to_string_lossy()]));

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "envsync_le_check",
            "arguments": { "path": tree.path().to_string_lossy() },
        },
    });
    let mut child = Command::new(BINARY)
        .arg("mcp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the server starts");
    writeln!(child.stdin.as_mut().expect("stdin"), "{request}").expect("written");
    let output = child.wait_with_output().expect("finishes");
    let response: serde_json::Value = serde_json::from_slice(
        output
            .stdout
            .split(|byte| *byte == b'\n')
            .next()
            .expect("a line"),
    )
    .expect("the reply is JSON");

    assert_eq!(
        response["result"]["structuredContent"]["data"], from_cli,
        "the two surfaces disagree"
    );
}
