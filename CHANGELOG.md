# Changelog

All notable changes to EnvSync-LE will be documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.1.0] - 2026-08-04

### Added

- Runtime strings are localized, and this time they render. All 6 of them —
  notifications, status bar, quick-picks and prompts — go through
  `vscode.l10n` and ship as twelve translated bundles in `l10n/`. The v1.x
  line carried manifest catalogues that worked and runtime catalogues that
  never reached the screen: `vscode-nls` was configured without
  `__filename`, so every runtime string fell back to English while the VSIX
  looked correct.
- An integration test covering both localization mechanisms — manifest
  substitution, key parity across all thirteen catalogues, and placeholder
  integrity in every translation. A translation that silently drops `{0}`
  now fails the build instead of shipping a message with the value missing.

- Dependency review on pull requests, failing on a high-severity addition
  before Dependabot's auto-merge can act.

### Fixed

- Twenty-four user-facing strings were never localized. This repo routes
  notifications through a `UserInterface` port rather than the notifier the
  other nine use, so `ui.showWarningMessage(...)` was a channel no earlier
  localization pass inspected; the status-bar tooltips and the missing-key,
  extra-key and parse-error message builders were missed for the same reason.
  Runtime localization now covers the whole surface — 6 strings before this
  release, 24 after.

### Fixed

- Eight more user-facing strings were never localized — error messages and a
  confirmation prompt built as template literals, which no property-based pass
  reaches. This repo routes notifications through a `UserInterface` port
  rather than the notifier the other nine use, which is why they were missed
  twice.
- The initial sync check in `activate` carried an unreachable error handler.
  `checkSync` resolves with an error report rather than rejecting — it catches
  internally and routes failures through `handleSyncCheckError`, which already
  notifies the user and honours `notificationLevel` — so the handler could not
  fire, and if it ever had it would have reported the same failure a second
  time behind its own duplicate level check.

### Changed

- Test coverage raised from 73.27% to 84.59% of branches (82.27% to 90.25% of
  statements). Five files sat below one of the repo's own floors; none do now.
  The gap was the three workspace commands — each a chain of file discovery, a
  quick pick and a comparison, where every step is reachable only by answering
  the one before it — and the file watcher, whose debounce, in-flight guard and
  error handling were never exercised because the suite registered it and
  never fired an event. The commands take their VS Code surface through ports,
  so they are driven with fakes rather than global mock state.


- CI gains fleet-wide checks that no single repo can perform: shared config is
  compared across all ten extensions, and every README link is verified —
  including Open VSX links, which are checked against the API because
  open-vsx.org answers HTTP 200 for extensions that do not exist.

## [2.0.1] - 2026-08-04

### Changed

- Marketplace categories re-targeted for discovery. `Other` is dropped
  (65,992 extensions, no discovery value); each extension now sits in
  categories matching how it is actually used.
- Search keywords widened to 30, targeting the terms users actually type
  rather than internal vocabulary.
- Toolchain moved to current: TypeScript 7, vitest 4, Biome 2.5.7,
  @types/node 26. `@types/vscode` is now pinned exactly to the
  `engines.vscode` floor — the caret had let the type surface drift 15
  minors ahead of the version actually supported.
- Runtime dependencies updated across majors where present: csv-parse 7,
  ini 7, js-yaml 5. Extraction output is unchanged, verified against the
  characterization goldens.
- Packaging no longer walks the npm tree (`vsce package --no-dependencies`).
  The bundle is self-contained, so the walk served no purpose and failed
  after any dependency change. Scrape-LE keeps it, since it genuinely
  ships `playwright-core`.
- Documentation claims corrected against the code. Removed: Numbers-LE
  "with statistics", EnvSync-LE "visual diffs", Regex-LE "live feedback",
  String-LE "and validation" — none of those features exist.

### Added

- Rating links in the in-extension help output, for both the VS Code
  Marketplace and Open VSX. Acquisitions exceed listing page views, so most
  users never see the listing's rating control; help is the surface they do
  reach.
- README now carries measured Performance and Testing sections, both
  generated rather than written — from `scripts/benchmark.ts` and from the
  coverage summary. CI fails if the coverage numbers drift from a real run.
- Coverage thresholds enforced at 75 lines / 80 functions / 60 branches /
  75 statements.
- CodeQL scanning, Dependabot with grouped weekly updates, and auto-merge
  limited to patch and minor devDependency bumps that pass CI.

## [2.0.0] - 2026-07-29

Full rehabilitation release. The headline: **v1.x VSIXes built from this
repo could not activate** — the build had no bundler while the package
excluded `node_modules`, so the extension crashed on load with
`Cannot find module 'vscode-nls'`. 2.0.0 ships a self-contained esbuild
bundle, verified by a packaging gate and a real extension-host
integration suite on every CI run.

### Fixed

- **Packaging**: `dist/extension.js` is now a single self-contained
  bundle (VSIX: 59 files → 21, no more stray `.claude/` settings in the
  package). A bundle gate (static require scan + loading the bundle
  with `vscode` stubbed) blocks any regression.
- **Context menu**: a file named exactly `.env` matched neither half of
  the when-clause (`resourceExtname == .env` — dotfiles have no
  extension — nor `/^\.env\./`), so the right-click menu never appeared
  on the primary file. Now one `resourceFilename` regex covering
  `.env`, `.env.*`, and `*.env` — and no longer `.envrc`.
- **Status bar**: the adapter was constructed without its configuration
  and read a hardcoded stub — `statusBar.enabled` had no effect at all.
  It now reads real config and reacts to changes without reload.
- **Watcher**: config was snapshotted once at activation; changes to
  `watchPatterns`, `excludePatterns`, `debounceMs`, or
  `notificationLevel` did nothing until a window reload.
- **Exclude patterns**: globs were anchored against the whole relative
  path, so `.env.*.local` never matched files in subdirectories.
  Patterns without `/` now match the basename (gitignore-style).
- **Config**: code fallbacks now provably match manifest defaults
  (asserted by a parity test) — `excludePatterns` silently disagreed;
  a non-numeric `debounceMs` no longer produces a `NaN` debounce; the
  string `"yes"` no longer reads as boolean `true`; the never-declared
  `notificationsLevel` typo key is no longer consulted.
- **Multi-select Quick Pick**: `Compare Selected` from the palette
  always behaved as cancelled — the adapter read `.label` off the
  returned *array*. Multi-pick now resolves to the picked values.
- **Compare Selected**: parse errors were silently discarded via an
  unreachable branch; they now appear in the report like everywhere else.
- **Keybinding docs**: help claimed `Ctrl+Alt+E`; the actual binding is
  `Ctrl+Alt+S`.

### Changed — detection output

- **`export KEY=value` now parses** (was an "Invalid key format" error).
- **Quoted values spanning lines** (`"`, `'`, or backtick) are consumed;
  their continuation lines were previously reported as "Missing equals
  sign" errors. An unterminated quote is now reported.
- **Duplicate keys count once** (were emitted twice, inflating counts).
- **File-type classification is segment-based**: `app.device.env` is no
  longer "development", `foo.test.env` no longer "test"; Windows paths
  classify correctly.
- **Extra keys are real in template mode**: files with keys the template
  lacks now produce the `extra-keys` status, count in the status bar,
  and notify at `notificationLevel: all` — all previously dead paths.
  In auto mode extras don't exist (the reference is the union).
- **`caseSensitive` is actually wired**: `false` now compares keys
  case-insensitively (it was declared and read but never used).
- Keys are extracted best-effort: a file with one malformed line still
  contributes its parseable keys alongside the errors.

### Removed

- 18 settings that were never read by any code path (`safety.*`,
  `performance.*`, `keyboard.*`, `presets.*`, `ignoreComments`).
  12 real settings remain, each with a live consumer.
- The runtime `vscode-nls` layer (it never worked — no bundles were
  ever generated; users saw inline English in every locale). Manifest
  translations (13 locales) are unaffected and now in exact key parity.
- 1,300+ lines of never-imported code (`utils/errorHandling.ts`,
  `utils/performance.ts`, `utils/safety.ts`, workspace adapter, unused
  mocks and fixtures), the unused `dotenv` dependency, `docs/` (its
  performance tables and governance documents did not correspond to
  this codebase), and `.cursorrules`.

### Added

- esbuild toolchain with bundle gate; tsc as typechecker covering tests.
- Characterization goldens pinning parser/classifier/glob/comparator
  output.
- Stateful vscode mock and a 127-test unit suite (85% line coverage,
  thresholds enforced) plus a real extension-host integration suite.
- CI on ubuntu/macos/windows and a manual release workflow
  (Marketplace + Open VSX).
- Error messages redact home directories and credential-shaped
  fragments.

## [1.8.1] and earlier - 2025

Historical releases, condensed: initial public release (1.7.0),
README/marketplace-link updates (1.8.0, 1.8.1). Their feature claims
did not all hold against the code — none of the packaged VSIXes could
activate (see 2.0.0), extra-key detection and several documented
settings were inert, and the published performance metrics documents
had no corresponding benchmark code. 2.0.0 is the first verifiable
release.
