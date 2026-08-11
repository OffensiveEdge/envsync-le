//! What counts as a dotenv file, and what kind it is.
//!
//! One place, ported from `heuristics.ts`, which is already the single
//! source the extension's parser, watcher and commands share — they used
//! to each carry their own copy and disagree.

use std::sync::LazyLock;

use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum FileType {
    Base,
    Local,
    Example,
    Production,
    Development,
    Test,
}

pub(crate) fn basename(filepath: &str) -> &str {
    let normalised = filepath.rsplit(['/', '\\']).next();
    normalised.unwrap_or(filepath)
}

pub(crate) fn is_env_file(filepath: &str) -> bool {
    let name = basename(filepath);
    // Case-sensitive on purpose. `.ENV` is not a name anyone writes, and
    // the extension compares exactly this way — matching it loosely here
    // would make the two frontends disagree about which files exist.
    name == ".env" || name.starts_with(".env.") || name.ends_with(".env")
}

/// Classify on **segments of the basename, not substrings**.
///
/// `.env.production` is production; `app.device.env` is **not**
/// development, which substring matching used to say it was. A
/// suffix-style name like `foo.env` is always base — its segments are
/// somebody else's naming scheme, not this one's.
///
/// When several segments match, the priority is local > example >
/// production > development > test, so `.env.production.local` is a
/// local override rather than a production file.
pub(crate) fn detect_file_type(filepath: &str) -> FileType {
    let name = basename(filepath);
    if name != ".env" && !name.starts_with(".env.") {
        return FileType::Base;
    }

    let segments: Vec<&str> = name.split('.').skip(2).collect();
    let has = |segment: &str| segments.contains(&segment);

    if has("local") {
        FileType::Local
    } else if has("example") || has("template") {
        FileType::Example
    } else if has("production") || has("prod") {
        FileType::Production
    } else if has("development") || has("dev") {
        FileType::Development
    } else if has("test") {
        FileType::Test
    } else {
        FileType::Base
    }
}

/// Whether an exclude pattern hides this file.
///
/// A pattern with no `/` matches the basename, gitignore-style, so
/// `.env.*.local` excludes nested files too. A pattern with a `/`
/// matches the workspace-relative path, and a leading `**/` matches zero
/// or more directories.
pub(crate) fn should_exclude(filepath: &str, patterns: &[String]) -> bool {
    let path = filepath.replace('\\', "/");
    let name = basename(&path).to_string();

    patterns.iter().any(|pattern| {
        let target = if pattern.contains('/') { &path } else { &name };
        glob_to_regex(pattern).is_match(target)
    })
}

fn glob_to_regex(pattern: &str) -> Regex {
    // `**/` first, so it can match zero directories as well as many;
    // translating it after `*` would leave a `/` that must be there.
    let mut source = String::from("^");
    let mut chars = pattern.chars().peekable();
    while let Some(character) = chars.next() {
        match character {
            '*' if chars.peek() == Some(&'*') => {
                chars.next();
                if chars.peek() == Some(&'/') {
                    chars.next();
                    source.push_str("(?:.*/)?");
                } else {
                    source.push_str(".*");
                }
            }
            '*' => source.push_str("[^/]*"),
            '?' => source.push_str("[^/]"),
            other => source.push_str(&regex::escape(&other.to_string())),
        }
    }
    source.push('$');
    Regex::new(&source).unwrap_or_else(|_| EVERYTHING.clone())
}

/// A pattern that cannot be compiled excludes nothing rather than
/// everything: a typo in a config should not silently hide the files the
/// tool exists to compare.
static EVERYTHING: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\b$").expect("a constant pattern compiles"));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_dotenv_names_are_recognised() {
        for name in [
            ".env",
            ".env.local",
            ".env.production",
            "foo.env",
            "a/b/.env",
        ] {
            assert!(is_env_file(name), "{name}");
        }
    }

    #[test]
    fn other_names_are_not() {
        for name in ["config.txt", ".environment", "env", "envrc", ".env-backup"] {
            assert!(!is_env_file(name), "{name}");
        }
    }

    #[test]
    fn a_segment_decides_the_type() {
        assert_eq!(detect_file_type(".env"), FileType::Base);
        assert_eq!(detect_file_type(".env.local"), FileType::Local);
        assert_eq!(detect_file_type(".env.example"), FileType::Example);
        assert_eq!(detect_file_type(".env.template"), FileType::Example);
        assert_eq!(detect_file_type(".env.production"), FileType::Production);
        assert_eq!(detect_file_type(".env.prod"), FileType::Production);
        assert_eq!(detect_file_type(".env.development"), FileType::Development);
        assert_eq!(detect_file_type(".env.dev"), FileType::Development);
        assert_eq!(detect_file_type(".env.test"), FileType::Test);
    }

    /// The bug that segment matching exists to prevent.
    #[test]
    fn a_substring_does_not_decide_the_type() {
        assert_eq!(detect_file_type("app.device.env"), FileType::Base);
        assert_eq!(detect_file_type("foo.env"), FileType::Base);
    }

    #[test]
    fn a_local_override_is_local_whatever_else_it_says() {
        assert_eq!(detect_file_type(".env.production.local"), FileType::Local);
        assert_eq!(detect_file_type(".env.test.local"), FileType::Local);
    }

    #[test]
    fn a_directory_does_not_change_the_type() {
        assert_eq!(
            detect_file_type("packages/api/.env.production"),
            FileType::Production
        );
    }

    #[test]
    fn a_bare_pattern_matches_the_basename_anywhere() {
        let patterns = vec![".env.*.local".to_string()];
        assert!(should_exclude(".env.production.local", &patterns));
        assert!(should_exclude("deep/nested/.env.test.local", &patterns));
        assert!(!should_exclude(".env.production", &patterns));
    }

    #[test]
    fn a_pattern_with_a_slash_matches_the_path() {
        let patterns = vec!["vendor/.env".to_string()];
        assert!(should_exclude("vendor/.env", &patterns));
        assert!(!should_exclude("app/.env", &patterns));
    }

    #[test]
    fn a_leading_double_star_matches_zero_or_more_directories() {
        let patterns = vec!["**/fixtures/.env".to_string()];
        assert!(should_exclude("fixtures/.env", &patterns));
        assert!(should_exclude("a/b/fixtures/.env", &patterns));
        assert!(!should_exclude("fixtures/nested/.env", &patterns));
    }

    #[test]
    fn a_star_does_not_cross_a_directory_boundary() {
        let patterns = vec!["a/*/.env".to_string()];
        assert!(should_exclude("a/b/.env", &patterns));
        assert!(!should_exclude("a/b/c/.env", &patterns));
    }

    /// A typo in a config must not silently hide the files this exists
    /// to compare.
    #[test]
    fn an_uncompilable_pattern_excludes_nothing() {
        let patterns = vec!["[".to_string()];
        assert!(!should_exclude(".env", &patterns));
    }

    #[test]
    fn no_patterns_exclude_nothing() {
        assert!(!should_exclude(".env", &[]));
    }
}
