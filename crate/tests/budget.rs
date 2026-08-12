//! A wall-clock ceiling, and a shape check on how the clock moves.
//!
//! A sibling crate was fifty times slower than the rest of the family
//! for a whole release and nothing noticed, because nothing measured it.
//! The ceiling here is deliberately loose — a shared runner is not a
//! benchmark rig — and exists to catch an order of magnitude, not a
//! percent.
//!
//! The second assertion is the one that catches the real class:
//! **the same tree four times over must not cost six times as much.**
//! A comparison is a set question, and the naive way to answer it is
//! quadratic in the number of files.
//!
//! Gated behind `ENVSYNC_LE_BUDGET` and run by CI on one platform with
//! `--test-threads=1`; a timing assertion measured against six other
//! tests on the same cores is noise. A skipped run says so by name.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const BINARY: &str = env!("CARGO_BIN_EXE_envsync-le");

/// The tree: 500 files, 40 keys each, one directory per service, every
/// tenth file short of a key so the comparison has real work to do.
/// Generated rather than checked in — 500 dotenv files in git would be
/// 500 files a reviewer has to ignore — and generated **from a fixed
/// seed**, so two runs measure the same tree.
const SEED: u64 = 0x0e5c_7a11_0b1d_2026;
const FILES: usize = 500;
const KEYS_PER_FILE: usize = 40;

/// **10× the local measurement**, recorded with the machine it came
/// from: 55 ms for this tree on an Apple M-series laptop, debug build,
/// 2026-08. Ten times that leaves a shared runner room to be several
/// times slower and still be right; it does not leave room for an order
/// of magnitude, which is the thing worth catching.
const BUDGET: Duration = Duration::from_millis(550);

/// Four times the tree, at most six times the clock. The slack absorbs
/// the walk's own super-linear moments (a bigger directory, a colder
/// cache); it does not absorb an O(files²) comparison.
const LINEARITY: f64 = 6.0;

fn enabled(name: &str) -> bool {
    if std::env::var_os("ENVSYNC_LE_BUDGET").is_some() {
        return true;
    }
    eprintln!("SKIPPED {name}: set ENVSYNC_LE_BUDGET to run it");
    false
}

/// A deterministic generator. No `rand` dependency: this needs a
/// reproducible tree, not statistical quality.
struct Seeded(u64);

impl Seeded {
    fn next(&mut self) -> u64 {
        // xorshift64*, chosen for being four lines and exactly
        // reproducible on every platform.
        let mut state = self.0;
        state ^= state >> 12;
        state ^= state << 25;
        state ^= state >> 27;
        self.0 = state;
        state.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }
}

struct Tree(PathBuf);

impl Tree {
    fn new(name: &str) -> Self {
        let root = std::env::temp_dir().join(format!("envsync-le-budget-{name}"));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("a temporary directory");
        Self(root)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Writes `FILES` dotenv files below `root`, deterministically.
fn populate(root: &Path, copies: usize) {
    let names = [".env", ".env.production", ".env.example", ".env.test"];
    for copy in 0..copies {
        let mut seeded = Seeded(SEED.wrapping_add(copy as u64));
        for index in 0..FILES {
            let service = index / 10;
            let name = names[index % names.len()];
            let mut body = String::new();
            for key in 0..KEYS_PER_FILE {
                // Every tenth file is short of one key, so the answer is
                // not trivially "all the same" and the reference lookup
                // has to run.
                if index % 10 == 0 && key == 7 {
                    continue;
                }
                let _ = writeln!(body, "SHARED_KEY_{key}=value-{}", seeded.next() % 1000);
            }
            // One directory per file: the names are the real dotenv
            // names, because a name the classifier does not recognise is
            // a file the walk never opens and a measurement of nothing.
            let target = root
                .join(format!("copy{copy}"))
                .join(format!("service-{service}"))
                .join(format!("unit-{index}"))
                .join(name);
            std::fs::create_dir_all(target.parent().expect("a parent")).expect("a directory");
            std::fs::write(&target, &body).expect("a file");
        }
    }
}

/// The fastest of three runs. The fastest, not the mean: a shared runner
/// pauses for reasons that have nothing to do with this code, and the
/// question is what the work costs, not what the neighbours cost.
fn fastest(root: &Path) -> Duration {
    let mut best = Duration::MAX;
    for _ in 0..3 {
        let started = Instant::now();
        let output = Command::new(BINARY)
            .arg(root)
            .stdin(Stdio::null())
            .output()
            .expect("the binary runs");
        let elapsed = started.elapsed();
        let code = output.status.code().expect("an exit code");
        assert!(
            (0..=1).contains(&code),
            "the measured run failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        best = best.min(elapsed);
    }
    best
}

#[test]
fn a_five_hundred_file_tree_scans_inside_its_budget() {
    if !enabled("a_five_hundred_file_tree_scans_inside_its_budget") {
        return;
    }
    let tree = Tree::new("single");
    populate(tree.path(), 1);

    let elapsed = fastest(tree.path());
    eprintln!("budget: {FILES} files in {elapsed:?} (ceiling {BUDGET:?}, seed {SEED:#x})");
    assert!(
        elapsed <= BUDGET,
        "{FILES} files took {elapsed:?}, over the {BUDGET:?} ceiling (seed {SEED:#x})"
    );
}

/// The quadratic detector. Every extra file is another file to compare
/// against, and the honest ways to do that are linear.
#[test]
fn four_times_the_tree_is_not_six_times_the_clock() {
    if !enabled("four_times_the_tree_is_not_six_times_the_clock") {
        return;
    }
    let single = Tree::new("linear-1");
    populate(single.path(), 1);
    let quadruple = Tree::new("linear-4");
    populate(quadruple.path(), 4);

    let one = fastest(single.path());
    let four = fastest(quadruple.path());
    let ratio = four.as_secs_f64() / one.as_secs_f64().max(f64::EPSILON);
    eprintln!(
        "budget: {one:?} for {FILES} files, {four:?} for {}, ratio {ratio:.2}",
        FILES * 4
    );
    assert!(
        ratio <= LINEARITY,
        "four times the tree cost {ratio:.2}× the time (limit {LINEARITY}×): \
         {one:?} for {FILES} files, {four:?} for {} (seed {SEED:#x})",
        FILES * 4
    );
}
