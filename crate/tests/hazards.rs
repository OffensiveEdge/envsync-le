//! Inputs a real repository contains and a fixture directory cannot.
//!
//! Every case here exists because something of this shape got through a
//! green suite somewhere in this family: a byte-order mark read as
//! content, a binary file turning a clean audit into exit 2, a file that
//! could not be decoded vanishing from the report entirely.
//!
//! The tree is **built at runtime**, not checked in: Windows cannot hold
//! a FIFO, a permission-denied file, or half of these names in git. Each
//! case the platform cannot express says so by name — see `skipped()` —
//! and a skip is never reported as a pass.
//!
//! Every case asserts the same floor: the process does not panic, does
//! not hang, and exits 0, 1 or 2 — never on a signal.

use std::fmt::Write as _;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

const BINARY: &str = env!("CARGO_BIN_EXE_envsync-le");

/// Generous enough for a shared runner reading a 1 MB line, tight enough
/// that a blocking read on a FIFO is a failure rather than a coffee
/// break.
const LIMIT: Duration = Duration::from_secs(60);

static COUNTER: AtomicUsize = AtomicUsize::new(0);

struct Tree {
    root: PathBuf,
}

impl Tree {
    fn new(name: &str) -> Self {
        let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "envsync-le-hazard-{name}-{}-{unique}",
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
        // Best effort: a chmod-000 file can outlive its directory on
        // some filesystems, and a failed cleanup must not fail a test.
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

struct Run {
    code: i32,
    stdout: String,
    stderr: String,
}

/// Runs the binary and **fails rather than blocks**. A hang is the
/// failure mode this file exists to catch — a FIFO with no writer is one
/// `read` away from an eternal CI job — so the child is killed and the
/// case is named.
fn run(case: &str, args: &[&str]) -> Run {
    let mut child = Command::new(BINARY)
        .args(args)
        // Never inherit the terminal: a child that reads stdin would
        // otherwise wait on a keyboard that is not there.
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the binary runs");

    // Drained on threads: a child writing more than a pipe buffer would
    // deadlock against a parent that waits before reading.
    let mut out = child.stdout.take().expect("stdout");
    let mut err = child.stderr.take().expect("stderr");
    let out = std::thread::spawn(move || {
        let mut buffer = Vec::new();
        let _ = out.read_to_end(&mut buffer);
        buffer
    });
    let err = std::thread::spawn(move || {
        let mut buffer = Vec::new();
        let _ = err.read_to_end(&mut buffer);
        buffer
    });

    let deadline = Instant::now() + LIMIT;
    let status = loop {
        match child.try_wait().expect("the child is waitable") {
            Some(status) => break status,
            None if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                panic!("{case}: hung for {LIMIT:?} on {args:?}");
            }
            None => std::thread::sleep(Duration::from_millis(10)),
        }
    };

    let code = status.code().unwrap_or_else(|| {
        panic!("{case}: died on a signal rather than exiting ({status:?}) on {args:?}")
    });
    assert!(
        (0..=2).contains(&code),
        "{case}: exit {code} is outside the documented 0/1/2 on {args:?}"
    );
    Run {
        code,
        stdout: String::from_utf8_lossy(&out.join().expect("stdout thread")).into_owned(),
        stderr: String::from_utf8_lossy(&err.join().expect("stderr thread")).into_owned(),
    }
}

fn report(case: &str, run: &Run) -> serde_json::Value {
    serde_json::from_str(run.stdout.trim())
        .unwrap_or_else(|error| panic!("{case}: stdout is not one JSON report ({error})"))
}

/// A case the platform cannot express. Named on stderr so a green run
/// still says what it did not check — a silent skip is a lie.
fn skipped(case: &str, why: &str) {
    eprintln!("SKIPPED {case}: {why}");
}

/// The keys a single-file tree reports.
fn keys_in(case: &str, tree: &Tree) -> u64 {
    let run = run(case, &[&tree.path().to_string_lossy()]);
    let report = report(case, &run);
    report["files"][0]["keys"].as_u64().unwrap_or_else(|| {
        panic!("{case}: no file in the report — {}", run.stdout);
    })
}

// ---------------------------------------------------------------- content

/// A BOM is three invisible bytes Notepad, Excel and a PowerShell
/// redirect all add. Left in, it attaches to the first key name and
/// every file looks out of sync with every other.
#[test]
fn a_byte_order_mark_does_not_change_the_keys() {
    let with = Tree::new("bom-with");
    with.write(".env", "\u{feff}KEY_ONE=1\nKEY_TWO=2\n");
    let without = Tree::new("bom-without");
    without.write(".env", "KEY_ONE=1\nKEY_TWO=2\n");

    let marked = run("bom", &[&with.path().to_string_lossy()]);
    let plain = run("bom", &[&without.path().to_string_lossy()]);
    assert_eq!(marked.code, plain.code);
    assert_eq!(
        report("bom", &marked)["files"][0]["keys"],
        report("bom", &plain)["files"][0]["keys"],
        "a BOM moved the key count"
    );
    assert_eq!(report("bom", &marked)["summary"]["files"], 1);
}

/// The same key, written on both sides of a BOM, must compare equal.
/// This is the shape of the bug: `\u{feff}KEY` and `KEY` reading as two
/// different names.
#[test]
fn a_byte_order_mark_does_not_invent_a_mismatch() {
    let tree = Tree::new("bom-compare");
    tree.write(".env", "\u{feff}SHARED=1\n");
    tree.write(".env.production", "SHARED=2\n");
    let run = run("bom-compare", &[&tree.path().to_string_lossy()]);
    assert_eq!(run.code, 0, "{}", run.stderr);
}

#[test]
fn line_endings_do_not_change_the_keys() {
    let crlf = Tree::new("crlf");
    crlf.write(".env", "A=1\r\nKEY_CRLF=2\r\n");
    assert_eq!(keys_in("crlf", &crlf), 2);

    // A lone CR is not a line ending here, and deliberately so: the
    // extension splits on /\r?\n/ and this splits on '\n', so both read
    // the whole thing as one entry. Pinned rather than improved — the
    // two frontends must agree.
    let cr = Tree::new("lone-cr");
    cr.write(".env", "A=1\rKEY_CR=2");
    assert_eq!(keys_in("lone-cr", &cr), 1);
}

#[test]
fn a_missing_trailing_newline_still_yields_its_key() {
    let tree = Tree::new("no-eol");
    tree.write(".env", "KEY_AT_EOF=1");
    assert_eq!(keys_in("no-eol", &tree), 1);
}

#[test]
fn an_empty_file_and_a_whitespace_file_are_clean() {
    for (case, body) in [("empty", ""), ("whitespace", "   \n\t\n \u{a0}\n")] {
        let tree = Tree::new(case);
        tree.write(".env", body);
        let run = run(case, &[&tree.path().to_string_lossy()]);
        assert_eq!(run.code, 0, "{case}: {}", run.stderr);
        let report = report(case, &run);
        assert_eq!(report["files"][0]["keys"], 0, "{case}");
        assert_eq!(
            report["diagnostics"].as_array().expect("a list").len(),
            0,
            "{case}: an empty file is not a diagnostic"
        );
    }
}

/// A NUL byte is valid UTF-8 and belongs to the line it sits on, so the
/// entry it breaks is a parse error and the keys below it survive.
#[test]
fn a_nul_byte_mid_file_breaks_one_line_and_no_more() {
    let tree = Tree::new("nul");
    tree.write(".env", "BEFORE=1\n\u{0}BROKEN=2\nAFTER=3\n");
    let run = run("nul", &[&tree.path().to_string_lossy()]);
    assert!(run.code <= 1, "{}", run.stderr);
    let report = report("nul", &run);
    assert_eq!(report["files"][0]["keys"], 2, "BEFORE and AFTER survive");
    assert!(
        report["diagnostics"]
            .as_array()
            .expect("a list")
            .iter()
            .any(|d| d["code"] == "parse-error"),
        "the broken line is named: {}",
        run.stdout
    );
}

/// **Never silently absent.** A file that is text but cannot be decoded
/// carries a `skipped` diagnostic and fails `--strict` — what is not
/// allowed is a file that vanishes from the report, which reads to
/// whoever ran it as a file that was clean.
#[test]
fn an_undecodable_file_is_reported_skipped_and_fails_strict() {
    for (case, bytes) in [
        ("invalid-utf8", vec![b'A', b'=', 0xff, 0xfe, b'\n']),
        // UTF-16LE with a BOM: what Notepad writes when asked for
        // "Unicode", and not UTF-8 by any reading.
        (
            "utf16le",
            vec![0xff, 0xfe, b'A', 0x00, b'=', 0x00, b'1', 0x00],
        ),
    ] {
        let tree = Tree::new(case);
        tree.write_bytes(".env", &bytes);
        tree.write(".env.production", "A=1\n");

        let lenient = run(case, &[&tree.path().to_string_lossy()]);
        assert!(
            lenient.code <= 1,
            "{case}: an unreadable file is not exit 2"
        );
        let report = report(case, &lenient);
        let diagnostics = report["diagnostics"].as_array().expect("a list");
        assert!(
            diagnostics.iter().any(|d| d["code"] == "skipped"),
            "{case}: no skipped diagnostic — {}",
            lenient.stdout
        );
        assert!(
            lenient.stderr.contains(".env"),
            "{case}: the skipped file is not named on stderr"
        );

        let strict = run(case, &["--strict", &tree.path().to_string_lossy()]);
        assert_eq!(strict.code, 2, "{case}: --strict tolerated a skipped file");
    }
}

/// A binary file is not a dotenv file, so it is not a report line and
/// not a `--strict` failure. secrets-le exited 2 on any repository
/// containing a PNG for a whole release.
#[test]
fn a_binary_file_is_not_a_report_line_and_not_a_strict_failure() {
    let tree = Tree::new("binary");
    // A PNG header, the exact case that broke a sibling crate.
    tree.write_bytes(
        "logo.png",
        &[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a],
    );
    tree.write(".env", "A=1\n");

    let lenient = run("binary", &[&tree.path().to_string_lossy()]);
    assert_eq!(lenient.code, 0, "{}", lenient.stderr);
    let report = report("binary", &lenient);
    assert_eq!(
        report["summary"]["files"], 1,
        "the PNG became a report line"
    );
    assert_eq!(report["diagnostics"].as_array().expect("a list").len(), 0);

    let strict = run("binary", &["--strict", &tree.path().to_string_lossy()]);
    assert_eq!(strict.code, 0, "a PNG must not fail --strict");
}

#[test]
fn a_four_byte_character_before_a_value_does_not_move_the_key() {
    let tree = Tree::new("emoji");
    tree.write(".env", "KEY_EMOJI=\u{1f389}celebrate\nKEY_AFTER=1\n");
    assert_eq!(keys_in("emoji", &tree), 2);
    let run = run("emoji", &[&tree.path().to_string_lossy()]);
    assert!(
        !run.stdout.contains("celebrate"),
        "a value reached stdout: {}",
        run.stdout
    );
}

#[test]
fn a_one_megabyte_line_completes() {
    let tree = Tree::new("long-line");
    let mut body = String::from("KEY_LONG=");
    body.push_str(&"x".repeat(1024 * 1024));
    body.push_str("\nKEY_AFTER=1\n");
    tree.write(".env", &body);
    assert_eq!(keys_in("long-line", &tree), 2);
}

#[test]
fn a_hundred_thousand_lines_complete() {
    let tree = Tree::new("many-lines");
    let mut body = String::new();
    for line in 0..100_000 {
        if line % 100 == 0 {
            let _ = writeln!(body, "KEY_{line}=1");
            continue;
        }
        body.push_str("# a comment line\n");
    }
    tree.write(".env", &body);
    assert_eq!(keys_in("many-lines", &tree), 1_000);
}

// ------------------------------------------------------------- filesystem

#[cfg(unix)]
fn symlink(original: &Path, link: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(original, link)
}

/// Windows needs Developer Mode or an elevated process to create one, so
/// a failure here is the platform refusing rather than the code being
/// wrong — the caller skips by name.
#[cfg(windows)]
fn symlink(original: &Path, link: &Path) -> std::io::Result<()> {
    if original.is_dir() {
        return std::os::windows::fs::symlink_dir(original, link);
    }
    std::os::windows::fs::symlink_file(original, link)
}

#[test]
fn symlinks_do_not_crash_the_walk() {
    let tree = Tree::new("symlink");
    tree.write(".env", "A=1\n");
    let target = tree.write("target.txt", "A=1\n");

    if symlink(&target, &tree.path().join(".env.link")).is_err() {
        skipped(
            "symlink-to-file",
            "this platform refused to create a symlink",
        );
        return;
    }
    let _ = symlink(
        &tree.path().join("nowhere-at-all"),
        &tree.path().join(".env.broken"),
    );

    let run = run("symlink", &[&tree.path().to_string_lossy()]);
    assert!(run.code <= 1, "{}", run.stderr);
    // The walk does not follow links, so a symlinked dotenv file is not
    // a file it reports. Pinned because the alternative — following —
    // is how a loop becomes an infinite walk.
    assert_eq!(
        report("symlink", &run)["summary"]["files"],
        1,
        "only the real .env is reported: {}",
        run.stdout
    );
}

#[test]
fn a_symlink_loop_terminates() {
    let tree = Tree::new("symlink-loop");
    tree.write(".env", "A=1\n");
    let first = tree.root.join("loop-a");
    let second = tree.root.join("loop-b");
    std::fs::create_dir_all(&first).expect("a directory");
    if symlink(&first, &second.join("back")).is_err() && symlink(&first, &second).is_err() {
        skipped("symlink-loop", "this platform refused to create a symlink");
        return;
    }
    let _ = symlink(&second, &first.join("forward"));

    let run = run("symlink-loop", &[&tree.path().to_string_lossy()]);
    assert!(run.code <= 1, "{}", run.stderr);
}

/// The hang this file exists for. A FIFO with no writer blocks a `read`
/// forever, and `--file` bypasses the walk's is-it-a-file guard.
#[cfg(unix)]
#[test]
fn a_fifo_does_not_block_the_run() {
    let tree = Tree::new("fifo");
    tree.write(".env", "A=1\n");
    let fifo = tree.path().join(".env.fifo");
    // Shelled out rather than called through libc: `unsafe` is forbidden
    // crate-wide and a test is not an exemption.
    let made = Command::new("mkfifo")
        .arg(&fifo)
        .status()
        .is_ok_and(|status| status.success());
    if !made {
        skipped("fifo", "mkfifo is not available on this runner");
        return;
    }

    let walked = run("fifo-walk", &[&tree.path().to_string_lossy()]);
    assert!(walked.code <= 1, "{}", walked.stderr);

    let named = run("fifo-named", &["--file", &fifo.to_string_lossy()]);
    assert!(
        named.code <= 2,
        "a named FIFO must refuse, not block: {}",
        named.stderr
    );
}

#[cfg(not(unix))]
#[test]
fn a_fifo_does_not_block_the_run() {
    skipped("fifo", "Windows has no FIFO in a directory tree");
}

/// An unreadable file is not a malformed question: it is one file in
/// fifty thousand, reported and left out of the exit code.
#[cfg(unix)]
#[test]
fn a_permission_denied_file_is_skipped_not_exit_two() {
    use std::os::unix::fs::PermissionsExt;

    let tree = Tree::new("denied");
    let denied = tree.write(".env.denied", "A=1\n");
    tree.write(".env", "A=1\n");
    std::fs::set_permissions(&denied, std::fs::Permissions::from_mode(0o000)).expect("chmod");
    if std::fs::read(&denied).is_ok() {
        skipped(
            "permission-denied",
            "this runner reads a mode-000 file anyway (root)",
        );
        return;
    }

    let lenient = run("denied", &[&tree.path().to_string_lossy()]);
    assert!(lenient.code <= 1, "an unreadable file became exit 2");
    assert!(
        report("denied", &lenient)["diagnostics"]
            .as_array()
            .expect("a list")
            .iter()
            .any(|d| d["code"] == "skipped"),
        "the unreadable file vanished from the report: {}",
        lenient.stdout
    );
    let strict = run("denied", &["--strict", &tree.path().to_string_lossy()]);
    assert_eq!(strict.code, 2);
}

#[cfg(not(unix))]
#[test]
fn a_permission_denied_file_is_skipped_not_exit_two() {
    skipped(
        "permission-denied",
        "Windows ACLs are not chmod; the unix case covers the read failure",
    );
}

/// A directory whose name is a dotenv name is still a directory.
#[test]
fn a_directory_named_like_a_dotenv_file_is_not_read() {
    let tree = Tree::new("dir-named-env");
    std::fs::create_dir_all(tree.root.join(".env.d")).expect("a directory");
    tree.write(".env.d/inner.txt", "not a dotenv file\n");
    tree.write(".env", "A=1\n");

    let run = run("dir-named-env", &[&tree.path().to_string_lossy()]);
    assert_eq!(run.code, 0, "{}", run.stderr);
    assert_eq!(report("dir-named-env", &run)["summary"]["files"], 1);
}

#[test]
fn awkward_file_names_are_found_and_reported_with_forward_slashes() {
    let tree = Tree::new("names");
    let names = [".env.with space", ".env.\u{fc}nicode", ".env.\u{1f389}"];
    let mut created = 0;
    for name in names {
        if std::fs::write(tree.root.join(name), "SHARED=1\n").is_ok() {
            created += 1;
        }
    }
    if created == 0 {
        skipped("awkward-names", "this filesystem refused every name");
        return;
    }
    tree.write(".env", "SHARED=1\n");

    let run = run("names", &[&tree.path().to_string_lossy()]);
    assert_eq!(run.code, 0, "{}", run.stderr);
    let report = report("names", &run);
    assert_eq!(report["summary"]["files"], created + 1);
    for file in report["files"].as_array().expect("a list") {
        let path = file["path"].as_str().expect("a path");
        assert!(
            !path.contains('\\'),
            "a backslash reached the report: {path}"
        );
    }
}

/// Where Windows differs: `MAX_PATH` is 260 characters unless long paths
/// are enabled, so the creation itself is the platform's answer.
#[test]
fn a_path_over_260_characters_does_not_crash_the_walk() {
    let tree = Tree::new("long-path");
    tree.write(".env", "SHARED=1\n");

    let mut deep = String::new();
    while deep.len() < 300 {
        deep.push_str("a-directory-with-a-long-name/");
    }
    deep.push_str(".env.deep");
    if std::fs::create_dir_all(tree.root.join(&deep).parent().expect("a parent")).is_err()
        || std::fs::write(tree.root.join(&deep), "SHARED=1\n").is_err()
    {
        skipped(
            "long-path",
            "this platform refused a path over 260 characters",
        );
        // The walk still has to survive the directories that were made.
        let run = run("long-path", &[&tree.path().to_string_lossy()]);
        assert!(run.code <= 1, "{}", run.stderr);
        return;
    }

    let run = run("long-path", &[&tree.path().to_string_lossy()]);
    assert_eq!(run.code, 0, "{}", run.stderr);
    assert_eq!(report("long-path", &run)["summary"]["files"], 2);
}

// ------------------------------------------------------------- exit codes

/// Exit 2 is for a malformed **question**, never for an awkward file.
#[test]
fn only_a_malformed_question_exits_two() {
    let tree = Tree::new("exit-two");
    tree.write(".env", "A=1\n");
    tree.write_bytes(".env.binary", &[0xff, 0xfe, 0x00]);

    assert!(
        run("exit-two", &[&tree.path().to_string_lossy()]).code <= 1,
        "an undecodable file is not a malformed question"
    );
    assert_eq!(
        run(
            "exit-two",
            &["--not-a-flag", &tree.path().to_string_lossy()]
        )
        .code,
        2
    );
    assert_eq!(run("exit-two", &["/no/such/place-xyz"]).code, 2);
}
