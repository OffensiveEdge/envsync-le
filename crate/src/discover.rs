//! Finding the dotenv files in a tree.
//!
//! **This is the only module allowed to touch the filesystem.** The
//! detection layer takes file contents and returns keys, which is what
//! lets the whole of it be tested from the corpus with no temp
//! directories and no flake.
//!
//! Directories are walked with ripgrep's `ignore`, so "what this tool
//! finds" and "what ripgrep finds" are the same answer. A file named
//! explicitly is always read, whatever the ignore rules say.
//!
//! Each crate in this family stands on its own: no shared crate, no
//! published core, and nothing holding this file equal to the similar
//! ones in the sibling repos. Where they agree it is because the same
//! answer was right twice; where they diverge that is the point.

use std::path::{Path as StdPath, PathBuf};

use crate::detect::compare::EnvFile;
use crate::detect::heuristics::{detect_file_type, is_env_file, should_exclude};
use crate::detect::parser::parse;

#[derive(Debug, Clone)]
pub(crate) struct DiscoverOptions {
    /// Descend hidden **directories**.
    ///
    /// Not the dotenv files themselves: `.env` is hidden by definition,
    /// so treating this the way the sibling crates do would make the
    /// default find nothing at all.
    pub(crate) hidden: bool,
    pub(crate) respect_ignore: bool,
    pub(crate) exclude: Vec<String>,
}

impl Default for DiscoverOptions {
    fn default() -> Self {
        Self {
            hidden: false,
            respect_ignore: true,
            exclude: Vec::new(),
        }
    }
}

/// The dotenv files under `root`, in a stable order.
///
/// The sort is not cosmetic: `ignore` makes no ordering guarantee, and a
/// report whose lines move between two runs over an unchanged tree
/// cannot be diffed — which is most of what a report in CI is for.
pub(crate) fn discover(root: &StdPath, options: &DiscoverOptions) -> Result<Vec<PathBuf>, String> {
    let metadata =
        std::fs::metadata(root).map_err(|error| format!("{}: {error}", root.display()))?;
    if metadata.is_file() {
        return Ok(vec![root.to_path_buf()]);
    }

    let mut builder = ignore::WalkBuilder::new(root);
    builder
        // A dotenv file is a dotfile, so the walk must see dotfiles at
        // all times; `hidden` here decides whether it descends hidden
        // *directories*, which is the question a caller actually has.
        .hidden(false)
        .git_ignore(options.respect_ignore)
        .git_global(options.respect_ignore)
        .git_exclude(options.respect_ignore)
        .ignore(options.respect_ignore)
        .parents(options.respect_ignore)
        .follow_links(false)
        .filter_entry({
            let descend_hidden = options.hidden;
            move |entry| {
                descend_hidden
                    || entry.file_type().is_none_or(|kind| !kind.is_dir())
                    || !is_hidden_directory(entry.path())
            }
        });

    let mut files = Vec::new();
    for entry in builder.build() {
        let entry = entry.map_err(|error| format!("{}: {error}", root.display()))?;
        if !entry.file_type().is_some_and(|kind| kind.is_file()) {
            continue;
        }
        let path = entry.into_path();
        let relative = path.strip_prefix(root).unwrap_or(&path).to_string_lossy();
        if !is_env_file(&relative) || should_exclude(&relative, &options.exclude) {
            continue;
        }
        files.push(path);
    }

    files.sort();
    files.dedup();
    Ok(files)
}

fn is_hidden_directory(path: &StdPath) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with('.') && name != "." && name != "..")
}

/// Read one file into the shape the comparator takes.
pub(crate) fn read(path: &PathBuf, root: &StdPath) -> Result<EnvFile, String> {
    let bytes = std::fs::read(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let content =
        String::from_utf8(bytes).map_err(|_| format!("{}: not UTF-8 text", path.display()))?;
    let content = without_bom(&content);
    let relative = path
        .strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .into_owned();
    let parsed = parse(content);
    Ok(EnvFile {
        kind: detect_file_type(&relative),
        path: relative,
        keys: parsed.keys,
        errors: parsed.errors,
    })
}

/// Drop a leading byte-order mark.
///
/// No editor shows it and VS Code strips it before the extension ever
/// sees a document, so without this the two frontends read the same file
/// differently the moment anything on Windows saves it — Notepad, Excel,
/// a PowerShell redirect. Here it would attach itself to the first key
/// name, so `\u{feff}API_URL` and `API_URL` would read as two different
/// keys and every file would look out of sync with every other.
pub(crate) fn without_bom(content: &str) -> &str {
    content.strip_prefix('\u{feff}').unwrap_or(content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::TempTree;

    fn names(files: &[PathBuf]) -> Vec<String> {
        files
            .iter()
            .map(|path| {
                path.file_name()
                    .expect("a file name")
                    .to_string_lossy()
                    .into_owned()
            })
            .collect()
    }

    #[test]
    fn the_dotenv_files_are_found_and_nothing_else_is() {
        let tree = TempTree::new("discover-basic");
        tree.write(".env", "A=1");
        tree.write(".env.example", "A=");
        tree.write("README.md", "not a dotenv file");
        tree.write("config.json", "{}");
        let found = discover(tree.path(), &DiscoverOptions::default()).expect("walks");
        assert_eq!(names(&found), [".env", ".env.example"]);
    }

    /// The whole default depends on this. A dotenv file is a dotfile,
    /// and a walk that skipped dotfiles would find none of them.
    #[test]
    fn dotfiles_are_found_without_asking() {
        let tree = TempTree::new("discover-dotfiles");
        tree.write(".env.production", "A=1");
        let found = discover(tree.path(), &DiscoverOptions::default()).expect("walks");
        assert_eq!(names(&found), [".env.production"]);
    }

    /// `--hidden` is about directories, which is the question a caller
    /// actually has.
    #[test]
    fn a_hidden_directory_is_skipped_unless_asked_for() {
        let tree = TempTree::new("discover-hidden-dir");
        tree.write(".env", "A=1");
        tree.write(".hidden/.env.production", "A=1");

        let default = discover(tree.path(), &DiscoverOptions::default()).expect("walks");
        assert_eq!(names(&default), [".env"]);

        let all = discover(
            tree.path(),
            &DiscoverOptions {
                hidden: true,
                ..DiscoverOptions::default()
            },
        )
        .expect("walks");
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn a_named_file_is_the_whole_walk() {
        let tree = TempTree::new("discover-one");
        let file = tree.write(".env", "A=1");
        let found = discover(&file, &DiscoverOptions::default()).expect("walks");
        assert_eq!(names(&found), [".env"]);
    }

    #[test]
    fn ignored_files_are_skipped() {
        let tree = TempTree::new("discover-ignore");
        tree.mkdir(".git");
        tree.write(".gitignore", ".env\n");
        tree.write(".env", "A=1");
        tree.write(".env.example", "A=");
        let found = discover(tree.path(), &DiscoverOptions::default()).expect("walks");
        assert_eq!(names(&found), [".env.example"]);
    }

    #[test]
    fn an_exclude_pattern_hides_a_file() {
        let tree = TempTree::new("discover-exclude");
        tree.write(".env", "A=1");
        tree.write(".env.production.local", "A=1");
        let found = discover(
            tree.path(),
            &DiscoverOptions {
                exclude: vec![".env.*.local".to_string()],
                ..DiscoverOptions::default()
            },
        )
        .expect("walks");
        assert_eq!(names(&found), [".env"]);
    }

    #[test]
    fn the_order_is_stable_between_runs() {
        let tree = TempTree::new("discover-order");
        for name in [".env.test", ".env", ".env.production"] {
            tree.write(name, "A=1");
        }
        let first = discover(tree.path(), &DiscoverOptions::default()).expect("walks");
        let again = discover(tree.path(), &DiscoverOptions::default()).expect("walks");
        assert_eq!(first, again);
        assert_eq!(names(&first), [".env", ".env.production", ".env.test"]);
    }

    #[test]
    fn a_missing_root_is_refused_by_name() {
        let tree = TempTree::new("discover-missing");
        let error = discover(&tree.path().join("nope"), &DiscoverOptions::default())
            .expect_err("a refusal");
        assert!(error.contains("nope"), "{error}");
    }

    #[test]
    fn reading_a_file_yields_its_keys_and_type() {
        let tree = TempTree::new("discover-read");
        let file = tree.write(".env.production", "A=1\nB=2\n");
        let read = read(&file, tree.path()).expect("reads");
        assert_eq!(read.path, ".env.production");
        assert_eq!(read.keys, ["A", "B"]);
        assert!(read.errors.is_empty());
    }

    #[test]
    fn a_path_is_reported_relative_to_the_root() {
        let tree = TempTree::new("discover-relative");
        let file = tree.write("packages/api/.env", "A=1");
        let read = read(&file, tree.path()).expect("reads");
        assert_eq!(read.path, "packages/api/.env");
    }
}
