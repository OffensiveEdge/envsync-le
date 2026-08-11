# Changelog

The Rust CLI and MCP server. The VS Code extension has its own
[CHANGELOG](../CHANGELOG.md) and its own version — the two products in
this repository release on their own cadence.

Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Versioning: [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-08-11

First release. The extension's detection engine, ported and pinned
against a shared corpus, over a tree instead of a pair of buffers.

### Added

- **The dotenv parser**, reproducing the extension's keys and parse
  errors for every case in `fixtures/` — including a quoted value that
  spans lines, and an unterminated quote swallowing the rest of the file
  the way dotenv itself does.
- **Filename classification** on segments rather than substrings, so
  `.env.production` is production and `app.device.env` is not
  development, with local winning when several segments match.
- **Both comparison modes.** Without a template the reference is the
  union of all keys and nothing can be extra; with one, that file is the
  contract and a key it lacks *is* extra — the question a deploy asks,
  and the one with no equivalent in a two-file diff.
- **Discovery** over a tree, `.gitignore` honoured, with glob excludes.
  `--hidden` controls directories: a dotenv file is a dotfile, so the
  walk always sees dotfiles.
- **The CLI**: one JSON report on stdout, a human summary on stderr, and
  exit codes — 0 in sync, 1 out of sync, 2 malformed question. No dotenv
  files at all is 0.
- **The MCP server** (`envsync-le mcp`) with two tools:
  `compare_env_files`, shared byte-for-byte with the npm server, and
  `envsync_le_check`.

### The shape of it

**It never reads a value.** A dotenv file is where credentials live, so
only key names are parsed, compared or reported. Three checks enforce it
rather than a promise: the parity script greps the corpus, a crate test
greps its embedded copy, and a contract test greps stdout and stderr of a
real run.

**The report carries no timestamp.** The extension's has one so it can
decide whether to re-render a status bar. A CLI has nothing to re-render,
and a timestamp makes every run differ from every other — which defeats
diffing a report against a baseline.

**Corpus documents are stored without their leading dot.**
`cargo package` skips dotfiles, so a corpus of `.env` files ships a crate
that cannot run its own tests. The corpus keeps the real names, because
classification reads them.

[0.1.0]: https://github.com/nolindnaidoo/envsync-le/releases/tag/crate-v0.1.0

### Fixed

- **A leading byte-order mark is no longer part of the document.** Three
  invisible bytes, added by Notepad, Excel and a PowerShell redirect, and
  stripped by VS Code before the extension ever sees a file — so the two
  frontends read the same file differently. It shifted every column on
  line one, and before a `{` it made a structured parser reject the whole
  document, which is indistinguishable from a file with no keys in it.

- **A file that cannot be read no longer fails the run.** Every
  repository has a PNG, a zip and something the runner lacks permission
  for. Exiting 2 on those made the tool unusable in CI, which is the one
  place it is most worth running. Such a file is now named on stderr and
  carried in the report with a `skipped` diagnostic, and the exit code
  reflects what was found. `--strict` restores the old behaviour for a
  pipeline that wants zero tolerance.

- **A file that is not text is named rather than dropped.** It used to
  vanish from the report entirely, which reads to whoever ran it as
  "that file was clean".
