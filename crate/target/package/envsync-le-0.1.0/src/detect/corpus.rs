//! The shared corpus, embedded and run as unit tests.
//!
//! `fixtures/` is the contract between this crate and the extension.
//! Embedding it means `cargo test` on the published tarball runs every
//! case, so whoever installed this can check the parity claim in the
//! README instead of trusting it.
//!
//! The extension's side is checked by
//! `../scripts/check-detection-parity.ts`.

/// One corpus document, by its **logical** name.
///
/// The names are `.env`, `.env.production` and so on, because
/// classification is one of the things being tested and it reads the
/// name. The files on disk drop the leading dot: `cargo package` skips
/// dotfiles, so a corpus of them would ship a crate that cannot run its
/// own tests — `cargo package` fails outright, which is how this was
/// found. The mapping lives here and in `../scripts/check-detection-parity.ts`.
///
/// Panics on a name the corpus does not carry — a test naming a file
/// that is not there is a broken test, not a runtime condition.
#[cfg_attr(not(test), expect(dead_code, reason = "the corpus is a test fixture"))]
pub(crate) fn document(name: &str) -> &'static str {
    match name {
        ".env" => include_str!("../../fixtures/documents/dot.env"),
        ".env.example" => include_str!("../../fixtures/documents/dot.env.example"),
        ".env.production" => include_str!("../../fixtures/documents/dot.env.production"),
        ".env.broken" => include_str!("../../fixtures/documents/dot.env.broken"),
        ".env.multiline" => include_str!("../../fixtures/documents/dot.env.multiline"),
        ".env.unterminated" => include_str!("../../fixtures/documents/dot.env.unterminated"),
        ".env.casing" => include_str!("../../fixtures/documents/dot.env.casing"),
        ".env.duplicate" => include_str!("../../fixtures/documents/dot.env.duplicate"),
        other => panic!("the corpus has no document named {other}"),
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;
    use serde_json::Value;

    use super::document;
    use crate::detect::compare::{EnvFile, Mode, Options, compare};
    use crate::detect::heuristics::{FileType, detect_file_type, is_env_file};
    use crate::detect::parser::{ParseError, parse};

    const CORPUS: &str = include_str!("../../fixtures/detection.json");

    #[derive(Debug, Deserialize)]
    struct Corpus {
        parsing: Vec<ParsingCase>,
        classification: Vec<ClassificationCase>,
        comparison: Vec<ComparisonCase>,
    }

    #[derive(Debug, Deserialize)]
    struct ParsingCase {
        file: String,
        keys: Vec<String>,
        errors: Vec<ParseError>,
    }

    #[derive(Debug, Deserialize)]
    struct ClassificationCase {
        path: String,
        #[serde(rename = "isEnv")]
        is_env: bool,
        #[serde(rename = "type")]
        kind: FileType,
    }

    #[derive(Debug, Deserialize)]
    struct ComparisonCase {
        name: String,
        files: Vec<String>,
        options: CaseOptions,
        expected: Value,
    }

    #[derive(Debug, Default, Deserialize)]
    struct CaseOptions {
        mode: Option<String>,
        #[serde(rename = "templatePath")]
        template_path: Option<String>,
        #[serde(rename = "caseSensitive")]
        case_sensitive: Option<bool>,
    }

    fn corpus() -> Corpus {
        serde_json::from_str(CORPUS).expect("the corpus is valid JSON")
    }

    fn as_file(name: &str) -> EnvFile {
        let parsed = parse(document(name));
        EnvFile {
            path: name.to_string(),
            kind: detect_file_type(name),
            keys: parsed.keys,
            errors: parsed.errors,
        }
    }

    #[test]
    fn every_corpus_document_parses_the_same() {
        let corpus = corpus();
        assert!(!corpus.parsing.is_empty(), "the corpus parses nothing");
        for case in corpus.parsing {
            let parsed = parse(document(&case.file));
            assert_eq!(parsed.keys, case.keys, "{} keys", case.file);
            assert_eq!(parsed.errors, case.errors, "{} errors", case.file);
        }
    }

    #[test]
    fn every_corpus_name_classifies_the_same() {
        let corpus = corpus();
        assert!(
            !corpus.classification.is_empty(),
            "the corpus classifies nothing"
        );
        for case in corpus.classification {
            assert_eq!(is_env_file(&case.path), case.is_env, "{} isEnv", case.path);
            assert_eq!(
                detect_file_type(&case.path),
                case.kind,
                "{} type",
                case.path
            );
        }
    }

    #[test]
    fn every_corpus_comparison_reproduces() {
        let corpus = corpus();
        assert!(!corpus.comparison.is_empty(), "the corpus compares nothing");
        for case in corpus.comparison {
            let files: Vec<EnvFile> = case.files.iter().map(|name| as_file(name)).collect();
            let template = case.options.template_path.clone();
            let options = Options {
                mode: match (case.options.mode.as_deref(), template.as_deref()) {
                    (Some("template"), Some(path)) => Mode::Template(path),
                    _ => Mode::Auto,
                },
                case_sensitive: case.options.case_sensitive.unwrap_or(true),
            };
            let actual =
                serde_json::to_value(compare(&files, options)).expect("a report serializes");
            assert_eq!(actual, case.expected, "{}", case.name);
        }
    }

    /// The property the whole crate rests on, checked against the corpus
    /// rather than trusted.
    #[test]
    fn no_corpus_answer_carries_a_value() {
        for secret in ["postgres://", "redis://", "sentry.io", "-----BEGIN-----"] {
            assert!(!CORPUS.contains(secret), "the corpus carries {secret}");
        }
    }
}
