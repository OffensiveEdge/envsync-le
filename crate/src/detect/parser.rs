//! The dotenv parser: key names, and the lines that are not keys.
//!
//! **Only names are read.** A dotenv file is where credentials live and
//! the answer does not need the values, so no value is ever kept, logged
//! or returned. The parity script greps the corpus for one.
//!
//! Ported from `parser.ts`, including the parts that look like bugs and
//! are documented behaviour.

use std::sync::LazyLock;

use regex::Regex;
use serde::{Deserialize, Serialize};

/// A key must look like an environment variable. Anything else to the
/// left of `=` is a parse error naming its line rather than a key.
static KEY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_-]*$").expect("a constant pattern compiles")
});

static EXPORT_PREFIX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^export\s+").expect("a constant pattern compiles"));

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ParseError {
    #[serde(rename = "type")]
    pub(crate) kind: String,
    pub(crate) message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct Parsed {
    pub(crate) keys: Vec<String>,
    pub(crate) errors: Vec<ParseError>,
}

pub(crate) fn parse(content: &str) -> Parsed {
    let lines: Vec<&str> = content
        .split('\n')
        .map(|line| line.trim_end_matches('\r'))
        .collect();
    let mut keys: Vec<String> = Vec::new();
    let mut errors = Vec::new();
    let mut index = 0;

    while index < lines.len() {
        let line = lines[index].trim();
        let line_number = index + 1;
        index += 1;

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let Some(equals) = line.find('=') else {
            errors.push(error(
                line_number,
                &format!("Missing equals sign in \"{line}\""),
            ));
            continue;
        };

        let key = EXPORT_PREFIX.replace(line[..equals].trim(), "").to_string();
        if key.is_empty() {
            errors.push(error(line_number, "Empty key before equals sign"));
            continue;
        }
        if !KEY.is_match(&key) {
            errors.push(error(line_number, &format!("Invalid key format \"{key}\"")));
            continue;
        }

        // A quoted value with no closing quote on this line spans the
        // following ones. Consuming them is what stops a certificate's
        // body being read as a page of malformed entries.
        if let Some(quote) = open_quote(line[equals + 1..].trim()) {
            while index < lines.len() && !closes_quote(lines[index], quote) {
                index += 1;
            }
            if index >= lines.len() {
                // dotenv's own behaviour: the rest of the file is inside
                // the value, so the keys below it are simply not there.
                // One stray quote can make a file look nearly empty, and
                // saying so is the whole point of reporting it.
                errors.push(error(
                    line_number,
                    &format!("Unterminated {quote} quoted value for \"{key}\""),
                ));
            } else {
                index += 1;
            }
        }

        // A duplicate counts once — dotenv itself keeps one occurrence.
        if !keys.iter().any(|existing| existing == &key) {
            keys.push(key);
        }
    }

    Parsed { keys, errors }
}

fn error(line: usize, message: &str) -> ParseError {
    ParseError {
        kind: "parse-error".to_string(),
        message: format!("Line {line}: {message}"),
    }
}

fn open_quote(value: &str) -> Option<char> {
    let first = value.chars().next()?;
    if !matches!(first, '"' | '\'' | '`') {
        return None;
    }
    let rest = &value[first.len_utf8()..];
    (!closes_quote(rest, first)).then_some(first)
}

fn closes_quote(line: &str, quote: char) -> bool {
    let mut chars = line.chars();
    while let Some(character) = chars.next() {
        if character == '\\' {
            chars.next();
            continue;
        }
        if character == quote {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys(content: &str) -> Vec<String> {
        parse(content).keys
    }

    #[test]
    fn a_plain_assignment_yields_its_key() {
        assert_eq!(keys("DATABASE_URL=postgres://x"), ["DATABASE_URL"]);
    }

    /// The property this whole crate rests on.
    #[test]
    fn a_value_is_never_kept() {
        let parsed = parse("SECRET=hunter2\nOTHER=postgres://user:pw@host/db");
        let rendered = format!("{parsed:?}");
        assert!(!rendered.contains("hunter2"), "{rendered}");
        assert!(!rendered.contains("postgres"), "{rendered}");
    }

    #[test]
    fn comments_and_blank_lines_are_skipped() {
        assert_eq!(keys("# a note\n\nA=1\n   \nB=2"), ["A", "B"]);
    }

    #[test]
    fn an_export_prefix_is_stripped() {
        assert_eq!(keys("export FEATURE_FLAGS=a,b"), ["FEATURE_FLAGS"]);
    }

    #[test]
    fn a_duplicate_counts_once() {
        assert_eq!(keys("DUP=1\nDUP=2\nOTHER=3"), ["DUP", "OTHER"]);
    }

    #[test]
    fn a_line_without_an_equals_is_an_error() {
        let parsed = parse("JUST_A_WORD");
        assert!(parsed.keys.is_empty());
        assert_eq!(parsed.errors.len(), 1);
        assert!(parsed.errors[0].message.contains("Missing equals sign"));
    }

    #[test]
    fn a_key_that_is_not_a_key_is_an_error() {
        let parsed = parse("2BAD=1\n=novalue\nGOOD=2");
        assert_eq!(parsed.keys, ["GOOD"]);
        assert_eq!(parsed.errors.len(), 2);
        assert!(parsed.errors[0].message.contains("Invalid key format"));
        assert!(parsed.errors[1].message.contains("Empty key"));
    }

    #[test]
    fn an_error_names_its_line() {
        let parsed = parse("A=1\nB=2\nBROKEN");
        assert!(
            parsed.errors[0].message.starts_with("Line 3:"),
            "{parsed:?}"
        );
    }

    /// A certificate in a `.env` is the ordinary case for this.
    #[test]
    fn a_quoted_value_may_span_lines() {
        let parsed = parse("FIRST=1\nCERT=\"-----BEGIN-----\nbody\n-----END-----\"\nAFTER=2");
        assert_eq!(parsed.keys, ["FIRST", "CERT", "AFTER"]);
        assert!(parsed.errors.is_empty());
    }

    #[test]
    fn every_quote_style_opens_a_multiline_value() {
        for quote in ['"', '\'', '`'] {
            let parsed = parse(&format!("A={quote}one\ntwo{quote}\nB=2"));
            assert_eq!(parsed.keys, ["A", "B"], "{quote}");
        }
    }

    /// dotenv's own behaviour, ported: the rest of the file is inside
    /// the value, so the keys below it are simply not there.
    #[test]
    fn an_unterminated_quote_swallows_the_rest_of_the_file() {
        let parsed = parse("BEFORE=1\nBROKEN=\"never closed\nAFTER=2\nLOST=3");
        assert_eq!(parsed.keys, ["BEFORE", "BROKEN"]);
        assert_eq!(parsed.errors.len(), 1);
        assert!(parsed.errors[0].message.contains("Unterminated"));
    }

    /// A quote that opens and closes on its own line is not multi-line.
    #[test]
    fn a_closed_quote_on_one_line_consumes_nothing() {
        assert_eq!(keys("A=\"one\"\nB=2"), ["A", "B"]);
    }

    #[test]
    fn an_escaped_quote_does_not_close_the_value() {
        let parsed = parse("A=\"one \\\" still open\ntwo\"\nB=2");
        assert_eq!(parsed.keys, ["A", "B"]);
    }

    #[test]
    fn carriage_returns_do_not_end_up_in_keys() {
        assert_eq!(keys("A=1\r\nB=2\r\n"), ["A", "B"]);
    }

    #[test]
    fn an_empty_document_yields_nothing() {
        assert_eq!(parse(""), Parsed::default());
    }
}
