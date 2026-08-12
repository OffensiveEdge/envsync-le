//! Behaviour that differs by operating system, asserted rather than
//! hoped.
//!
//! This crate shipped a release whose report used `\` on Windows and `/`
//! everywhere else. Nothing was red: the suite only ever ran the
//! separator it was written on. Every path this produces is checked here
//! on all three platforms, and the checks below are the rest of the
//! class — a case-folding filesystem, a name Windows will not create, a
//! stdin the child closes before the parent finishes writing.

use std::io::{Read, Write};
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
            "envsync-le-platform-{name}-{}-{unique}",
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
        self.write_bytes(relative, contents.as_bytes())
    }

    fn write_bytes(&self, relative: &str, contents: &[u8]) -> PathBuf {
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

fn run_with(args: &[&str], environment: &[(&str, Option<&str>)]) -> Run {
    let mut command = Command::new(BINARY);
    command.args(args).stdin(Stdio::null());
    for (name, value) in environment {
        match value {
            Some(value) => command.env(name, value),
            None => command.env_remove(name),
        };
    }
    let output = command.output().expect("the binary runs");
    Run {
        code: output.status.code().expect("an exit code, never a signal"),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

fn run(args: &[&str]) -> Run {
    run_with(args, &[])
}

fn report(run: &Run) -> serde_json::Value {
    serde_json::from_str(run.stdout.trim()).expect("stdout carries one JSON report")
}

/// Every string in the report that names a file, wherever it appears —
/// **including the free text of a diagnostic**, which is where the last
/// platform-native path was hiding after the obvious one was fixed.
fn paths_in(report: &serde_json::Value) -> Vec<String> {
    let mut found = Vec::new();
    for file in report["files"].as_array().expect("files") {
        found.push(file["path"].as_str().expect("a path").to_string());
    }
    for group in ["missingKeys", "extraKeys"] {
        for mismatch in report[group].as_array().expect("a list") {
            found.push(mismatch["filepath"].as_str().expect("a path").to_string());
            found.push(mismatch["reference"].as_str().expect("a name").to_string());
        }
    }
    for diagnostic in report["diagnostics"].as_array().expect("a list") {
        found.push(diagnostic["file"].as_str().expect("a path").to_string());
        found.push(
            diagnostic["message"]
                .as_str()
                .expect("a message")
                .to_string(),
        );
    }
    found
}

fn skipped(case: &str, why: &str) {
    eprintln!("SKIPPED {case}: {why}");
}

/// **The bug this file is named after.** The separator reaches the
/// report, so leaving it to the platform means the same repository
/// describes itself two ways and every diff between two machines shows a
/// change that is not one.
#[test]
fn every_path_in_the_report_uses_forward_slashes() {
    let tree = Tree::new("slashes");
    tree.write("packages/api/.env.example", "SHARED=\nONLY_HERE=\n");
    tree.write("packages/api/.env", "SHARED=1\nLOCAL=2\n");
    tree.write_bytes("packages/web/.env", &[b'A', b'=', 0xff, b'\n']);

    let run = run(&[
        "--template",
        "packages/api/.env.example",
        &tree.path().to_string_lossy(),
    ]);
    let report = report(&run);
    let paths = paths_in(&report);
    assert!(
        paths.len() >= 5,
        "the tree did not produce the paths under test: {}",
        run.stdout
    );
    // Not vacuous: the tree is nested on purpose, so at least one of
    // these has to carry a separator for the check below to mean
    // anything. An assertion that passes because there were no paths is
    // the failure mode this whole file exists to avoid.
    assert!(
        paths.iter().any(|path| path.contains('/')),
        "no path in the report carried a separator, so nothing was checked: {}",
        run.stdout
    );
    for path in &paths {
        assert!(
            !path.contains('\\'),
            "a backslash reached the report: {path}\n{}",
            run.stdout
        );
    }
    assert!(
        paths.iter().any(|path| path.contains("packages/api/.env")),
        "the nested path is not in the report: {}",
        run.stdout
    );
    // A file that could not be read is named the same way as one that
    // was: relative to the root. Reaching for the absolute path here is
    // how the backslash got in on Windows.
    let diagnostics = report["diagnostics"].as_array().expect("a list");
    assert!(
        diagnostics
            .iter()
            .any(|d| d["file"] == "packages/web/.env" && d["code"] == "skipped"),
        "the skipped file is not named relative to the root: {}",
        run.stdout
    );
    // stderr is a projection of the same report and carries the same
    // names, so it is held to the same rule.
    assert!(
        !run.stderr.contains("packages\\"),
        "a backslash reached stderr: {}",
        run.stderr
    );
}

/// The report carries no timestamp and reads no clock, so the answer
/// cannot depend on `TZ` — which matters because Windows ignores the
/// variable entirely and a suite that quietly depended on it would be
/// red there and nowhere else.
#[test]
fn the_report_does_not_depend_on_the_time_zone() {
    let tree = Tree::new("tz");
    tree.write(".env.example", "A=\nB=\n");
    tree.write(".env", "A=1\n");

    let root = tree.path().to_string_lossy().into_owned();
    let args = ["--template", ".env.example", &root];
    let utc = run_with(&args, &[("TZ", Some("UTC"))]);
    let unset = run_with(&args, &[("TZ", None)]);
    let far = run_with(&args, &[("TZ", Some("Pacific/Kiritimati"))]);

    assert_eq!(utc.stdout, unset.stdout, "TZ=UTC differs from TZ unset");
    assert_eq!(utc.stdout, far.stdout, "the report moved with the clock");
    assert_eq!(utc.code, unset.code);
    assert_eq!(utc.code, far.code);
}

/// `README.md` and `readme.md` are one file on macOS and Windows and two
/// on Linux. Whichever it is, the walk reports each file once.
#[test]
fn a_case_folding_filesystem_never_reports_a_file_twice() {
    let tree = Tree::new("case-fold");
    tree.write(".env.local", "A=1\n");
    tree.write(".env.LOCAL", "A=1\n");

    let on_disk = std::fs::read_dir(tree.path())
        .expect("a directory")
        .filter_map(Result::ok)
        .count();
    assert!(
        (1..=2).contains(&on_disk),
        "the case probe wrote {on_disk} files"
    );

    let run = run(&[&tree.path().to_string_lossy()]);
    let report = report(&run);
    let mut paths: Vec<String> = report["files"]
        .as_array()
        .expect("files")
        .iter()
        .map(|file| file["path"].as_str().expect("a path").to_string())
        .collect();
    assert_eq!(
        paths.len(),
        on_disk,
        "the walk reported {} files for {on_disk} on disk: {}",
        paths.len(),
        run.stdout
    );
    let before = paths.len();
    paths.sort();
    paths.dedup();
    assert_eq!(paths.len(), before, "a file was reported twice: {paths:?}");
}

/// `CON`, `PRN`, `AUX`, `NUL` and `COM1` are device names on Windows and
/// ordinary files everywhere else. The test asserts the walk **survives
/// the creation failing**, never that the files exist.
#[test]
fn reserved_windows_names_do_not_break_the_walk() {
    let tree = Tree::new("reserved");
    tree.write(".env", "SHARED=1\n");

    let mut created = Vec::new();
    for name in ["CON", "PRN", "AUX", "NUL", "COM1"] {
        for candidate in [name.to_string(), format!(".env.{name}")] {
            if std::fs::write(tree.path().join(&candidate), "SHARED=1\n").is_ok() {
                created.push(candidate);
            }
        }
    }
    if created.is_empty() {
        skipped("reserved-names", "this platform refused every device name");
    }

    let run = run(&[&tree.path().to_string_lossy()]);
    assert!(run.code <= 1, "{}", run.stderr);
    let report = report(&run);
    // Only `.env`-shaped names are dotenv files, so a bare `CON` is
    // never a report line even where it was created.
    assert!(
        report["summary"]["files"].as_u64().expect("a count") >= 1,
        "{}",
        run.stdout
    );
}

/// **Assert the exit code, never the write.** A child that refuses
/// before draining stdin closes the pipe under the parent's feet; a test
/// that asserted the write succeeded was red on one platform and green
/// on the others for reasons that had nothing to do with the code.
#[test]
fn a_child_that_refuses_early_does_not_fail_on_the_write() {
    let mut child = Command::new(BINARY)
        .arg("--not-a-flag")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the binary runs");

    // Deliberately more than a pipe buffer, and deliberately unchecked.
    if let Some(stdin) = child.stdin.as_mut() {
        let _ = stdin.write_all(&vec![b'x'; 1024 * 1024]);
        let _ = stdin.flush();
    }
    drop(child.stdin.take());

    let output = child.wait_with_output().expect("the child finishes");
    assert_eq!(
        output.status.code(),
        Some(2),
        "a malformed question is exit 2 whatever happened to stdin"
    );
}

/// The MCP server reads stdin to end of stream. Closing it immediately
/// is a client that went away, and the answer is a clean exit rather
/// than a broken pipe.
#[test]
fn the_mcp_server_exits_cleanly_when_stdin_closes_immediately() {
    let mut child = Command::new(BINARY)
        .arg("mcp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the binary runs");
    drop(child.stdin.take());

    let mut stdout = String::new();
    if let Some(pipe) = child.stdout.as_mut() {
        let _ = pipe.read_to_string(&mut stdout);
    }
    let status = child.wait().expect("the child finishes");
    assert_eq!(status.code(), Some(0), "stdout was {stdout:?}");
}
