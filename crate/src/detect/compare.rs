//! The comparison: which keys are missing from which file.
//!
//! Two modes that mean different things, and the difference is the
//! product:
//!
//! - **auto** — the reference is the union of every key in every file,
//!   so nothing can be *extra*; a key present anywhere must be present
//!   everywhere. This is the "these environments have drifted" question.
//! - **template** — one named file is the contract, and a key it does
//!   not have **is** extra. This is the "will this deploy break"
//!   question, and it is the one with no equivalent in a two-file diff.

use serde::{Deserialize, Serialize};

use super::heuristics::FileType;
use super::parser::ParseError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Status {
    InSync,
    MissingKeys,
    ExtraKeys,
    NoFiles,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EnvFile {
    pub(crate) path: String,
    pub(crate) kind: FileType,
    pub(crate) keys: Vec<String>,
    pub(crate) errors: Vec<ParseError>,
}

/// A file's place in the report: its name, its type, and **how many**
/// keys it has. The names appear only where they are the finding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct FileSummary {
    pub(crate) path: String,
    #[serde(rename = "type")]
    pub(crate) kind: FileType,
    pub(crate) keys: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct Mismatch {
    #[serde(rename = "filepath")]
    pub(crate) file: String,
    pub(crate) keys: Vec<String>,
    /// A file that does have these keys, or the literal `other files`
    /// when no single one does.
    pub(crate) reference: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mode<'a> {
    Auto,
    Template(&'a str),
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Options<'a> {
    pub(crate) mode: Mode<'a>,
    pub(crate) case_sensitive: bool,
}

impl Default for Options<'_> {
    fn default() -> Self {
        Self {
            mode: Mode::Auto,
            case_sensitive: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct Report {
    pub(crate) status: Status,
    pub(crate) files: Vec<FileSummary>,
    #[serde(rename = "missingKeys")]
    pub(crate) missing_keys: Vec<Mismatch>,
    #[serde(rename = "extraKeys")]
    pub(crate) extra_keys: Vec<Mismatch>,
    /// Always empty, and matching the extension in being so.
    ///
    /// `compareFiles` there returns a frozen empty array: parse errors
    /// reach the report from the detector, not from the comparison. The
    /// field is carried rather than dropped because the corpus compares
    /// this structure against the extension's, and a missing key is a
    /// difference. The CLI collects the real errors separately.
    pub(crate) errors: Vec<ParseError>,
}

pub(crate) fn compare(files: &[EnvFile], options: Options) -> Report {
    if files.is_empty() {
        return Report {
            status: Status::NoFiles,
            files: Vec::new(),
            missing_keys: Vec::new(),
            extra_keys: Vec::new(),
            errors: Vec::new(),
        };
    }

    let template = match options.mode {
        // A template that is not among the files falls back to the union
        // rather than failing. Naming a file that is not there is a
        // question about a tree that does not exist, and answering it
        // with every key present is more use than an error.
        // **Matched by the name it ends in, not by the spelling given.**
        // `file.path` is the discovered relative path, so a template
        // named `./.env.example`, `<abs>/.env.example` or
        // `sub/.env.example` compared unequal to `.env.example` and fell
        // through to the union — silently, with `"mode":"template"` still
        // in the report and `extraKeys` emptied, so the blame inverted
        // onto the contract file. Only one spelling of a present file
        // worked, and it was not the one a person types beside the
        // directory argument.
        Mode::Template(path) => files
            .iter()
            .find(|file| file.path == path)
            .or_else(|| files.iter().find(|file| same_file(&file.path, path))),
        Mode::Auto => None,
    };
    let reference: Vec<String> = match template {
        Some(file) => file.keys.clone(),
        None => union_of(files),
    };

    let missing_keys = find_missing(files, &reference, options.case_sensitive);
    // Extra keys exist only against a template: with the union as the
    // reference, nothing can be outside it by construction.
    let extra_keys = template.map_or_else(Vec::new, |template| {
        find_extra(files, &reference, template, options.case_sensitive)
    });

    Report {
        status: status_of(&missing_keys, &extra_keys),
        files: files
            .iter()
            .map(|file| FileSummary {
                path: file.path.clone(),
                kind: file.kind,
                keys: file.keys.len(),
            })
            .collect(),
        missing_keys,
        extra_keys,
        errors: Vec::new(),
    }
}

fn normalise(key: &str, case_sensitive: bool) -> String {
    if case_sensitive {
        key.to_string()
    } else {
        key.to_lowercase()
    }
}

fn union_of(files: &[EnvFile]) -> Vec<String> {
    let mut union: Vec<String> = Vec::new();
    for file in files {
        for key in &file.keys {
            if !union.contains(key) {
                union.push(key.clone());
            }
        }
    }
    union
}

/// Whether a discovered path and a named template are the same file.
///
/// Compared by trailing component rather than by canonicalising: the
/// discovered path is relative to the root that was walked, the named
/// one is whatever was typed, and neither is resolved against the
/// filesystem here — `compare` reads no files. A template that names
/// more than a base name must match that much of the tail, so
/// `config/.env.example` does not answer for `other/.env.example`.
fn same_file(discovered: &str, named: &str) -> bool {
    let discovered = discovered.trim_start_matches("./");
    let named = named.trim_start_matches("./");
    if discovered == named {
        return true;
    }
    // One is a suffix of the other on a component boundary.
    let (long, short) = if discovered.len() >= named.len() {
        (discovered, named)
    } else {
        (named, discovered)
    };
    long.strip_suffix(short)
        .is_some_and(|prefix| prefix.is_empty() || prefix.ends_with('/'))
}

fn find_missing(files: &[EnvFile], reference: &[String], case_sensitive: bool) -> Vec<Mismatch> {
    let mut mismatches = Vec::new();
    for file in files {
        let present: Vec<String> = file
            .keys
            .iter()
            .map(|key| normalise(key, case_sensitive))
            .collect();
        let missing: Vec<String> = reference
            .iter()
            .filter(|key| !present.contains(&normalise(key, case_sensitive)))
            .cloned()
            .collect();
        if missing.is_empty() {
            continue;
        }
        mismatches.push(Mismatch {
            file: file.path.clone(),
            reference: reference_for(files, file, &missing, case_sensitive),
            keys: missing,
        });
    }
    mismatches
}

/// Name a file that actually has the missing keys, so the reader has
/// somewhere to look. When no single file has all of them, the answer is
/// the literal `other files` rather than an arbitrary pick.
fn reference_for(
    files: &[EnvFile],
    subject: &EnvFile,
    missing: &[String],
    case_sensitive: bool,
) -> String {
    files
        .iter()
        .find(|candidate| {
            candidate.path != subject.path
                && missing.iter().all(|key| {
                    candidate.keys.iter().any(|have| {
                        normalise(have, case_sensitive) == normalise(key, case_sensitive)
                    })
                })
        })
        .map_or_else(|| "other files".to_string(), |file| file.path.clone())
}

fn find_extra(
    files: &[EnvFile],
    reference: &[String],
    template: &EnvFile,
    case_sensitive: bool,
) -> Vec<Mismatch> {
    let allowed: Vec<String> = reference
        .iter()
        .map(|key| normalise(key, case_sensitive))
        .collect();

    files
        .iter()
        .filter(|file| file.path != template.path)
        .filter_map(|file| {
            let extra: Vec<String> = file
                .keys
                .iter()
                .filter(|key| !allowed.contains(&normalise(key, case_sensitive)))
                .cloned()
                .collect();
            (!extra.is_empty()).then(|| Mismatch {
                file: file.path.clone(),
                keys: extra,
                reference: template.path.clone(),
            })
        })
        .collect()
}

fn status_of(missing: &[Mismatch], extra: &[Mismatch]) -> Status {
    if !missing.is_empty() {
        Status::MissingKeys
    } else if !extra.is_empty() {
        Status::ExtraKeys
    } else {
        Status::InSync
    }
}

#[cfg(test)]
mod same_file_tests {
    use super::same_file;

    /// Only one spelling of a present file used to match, and it was
    /// not the one a person types beside the directory argument. The
    /// failure was silent: `"mode":"template"` stayed in the report,
    /// `extraKeys` emptied, and the blame inverted onto the contract.
    #[test]
    fn a_template_matches_however_it_is_spelled() {
        assert!(same_file(".env.example", ".env.example"));
        assert!(same_file(".env.example", "./.env.example"));
        assert!(same_file(".env.example", "/abs/path/.env.example"));
        assert!(same_file("sub/.env", "sub/.env"));
        assert!(same_file("sub/.env", "/abs/sub/.env"));
    }

    /// A suffix must land on a component boundary, or `.env.example`
    /// would answer for `my.env.example` and a template would name a
    /// file nobody meant.
    #[test]
    fn a_partial_component_is_not_the_same_file() {
        assert!(!same_file(".env.example", "my.env.example"));
        assert!(!same_file("config/.env", "other/.env.example"));
        assert!(!same_file(".env", ".env.local"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(path: &str, keys: &[&str]) -> EnvFile {
        EnvFile {
            path: path.to_string(),
            kind: super::super::heuristics::detect_file_type(path),
            keys: keys.iter().map(|key| (*key).to_string()).collect(),
            errors: Vec::new(),
        }
    }

    #[test]
    fn nothing_to_compare_is_no_files() {
        assert_eq!(compare(&[], Options::default()).status, Status::NoFiles);
    }

    #[test]
    fn one_file_is_always_in_sync() {
        let report = compare(&[file(".env", &["A", "B"])], Options::default());
        assert_eq!(report.status, Status::InSync);
    }

    #[test]
    fn matching_files_are_in_sync() {
        let files = [
            file(".env", &["A", "B"]),
            file(".env.production", &["A", "B"]),
        ];
        assert_eq!(compare(&files, Options::default()).status, Status::InSync);
    }

    #[test]
    fn a_key_present_anywhere_must_be_present_everywhere() {
        let files = [file(".env", &["A"]), file(".env.production", &["A", "B"])];
        let report = compare(&files, Options::default());
        assert_eq!(report.status, Status::MissingKeys);
        assert_eq!(report.missing_keys[0].file, ".env");
        assert_eq!(report.missing_keys[0].keys, ["B"]);
        assert_eq!(report.missing_keys[0].reference, ".env.production");
    }

    /// The defining property of auto mode.
    #[test]
    fn nothing_can_be_extra_without_a_template() {
        let files = [
            file(".env", &["A", "EXTRA"]),
            file(".env.production", &["A"]),
        ];
        let report = compare(&files, Options::default());
        assert!(report.extra_keys.is_empty());
    }

    /// And the defining property of template mode.
    #[test]
    fn a_key_the_template_lacks_is_extra() {
        let files = [
            file(".env.example", &["A"]),
            file(".env", &["A", "LOCAL_ONLY"]),
        ];
        let report = compare(
            &files,
            Options {
                mode: Mode::Template(".env.example"),
                ..Options::default()
            },
        );
        assert_eq!(report.extra_keys[0].file, ".env");
        assert_eq!(report.extra_keys[0].keys, ["LOCAL_ONLY"]);
        assert_eq!(report.extra_keys[0].reference, ".env.example");
    }

    #[test]
    fn the_template_itself_is_never_extra_or_missing() {
        let files = [file(".env.example", &["A", "B"]), file(".env", &["A", "B"])];
        let report = compare(
            &files,
            Options {
                mode: Mode::Template(".env.example"),
                ..Options::default()
            },
        );
        assert_eq!(report.status, Status::InSync);
    }

    #[test]
    fn a_template_that_is_not_there_falls_back_to_the_union() {
        let files = [file(".env", &["A"]), file(".env.production", &["A", "B"])];
        let report = compare(
            &files,
            Options {
                mode: Mode::Template(".env.missing"),
                ..Options::default()
            },
        );
        assert_eq!(report.missing_keys[0].keys, ["B"]);
        assert!(report.extra_keys.is_empty(), "no template, nothing extra");
    }

    #[test]
    fn case_sensitivity_is_on_by_default() {
        let files = [
            file(".env", &["database_url"]),
            file(".env.production", &["DATABASE_URL"]),
        ];
        assert_eq!(
            compare(&files, Options::default()).status,
            Status::MissingKeys
        );
    }

    #[test]
    fn case_can_be_ignored() {
        let files = [
            file(".env", &["database_url"]),
            file(".env.production", &["DATABASE_URL"]),
        ];
        let report = compare(
            &files,
            Options {
                case_sensitive: false,
                ..Options::default()
            },
        );
        assert_eq!(report.status, Status::InSync);
    }

    /// When no single file has all the missing keys there is nowhere
    /// honest to point, and saying so beats picking one.
    #[test]
    fn a_reference_is_only_named_when_one_file_has_them_all() {
        let files = [
            file(".env", &[]),
            file(".env.a", &["A"]),
            file(".env.b", &["B"]),
        ];
        let report = compare(&files, Options::default());
        let first = &report.missing_keys[0];
        assert_eq!(first.file, ".env");
        assert_eq!(first.reference, "other files");
    }

    #[test]
    fn missing_beats_extra_in_the_status() {
        let files = [file(".env.example", &["A"]), file(".env", &["EXTRA"])];
        let report = compare(
            &files,
            Options {
                mode: Mode::Template(".env.example"),
                ..Options::default()
            },
        );
        assert!(!report.missing_keys.is_empty());
        assert!(!report.extra_keys.is_empty());
        assert_eq!(report.status, Status::MissingKeys);
    }

    /// The report counts keys rather than listing them; the names appear
    /// only where they are the finding.
    #[test]
    fn a_files_entry_carries_a_count_not_the_names() {
        let report = compare(&[file(".env", &["A", "B"])], Options::default());
        assert_eq!(report.files[0].keys, 2);
        let rendered = serde_json::to_string(&report).expect("serializes");
        assert!(!rendered.contains("\"A\""), "{rendered}");
    }
}
