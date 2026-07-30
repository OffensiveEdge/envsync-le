# AGENTS.md — EnvSync-LE

Technical source of truth for this repo. README.md is the user-facing doc; this file is for anyone (human or agent) changing the code.

## What this is

A VS Code extension that compares the variable names (keys) across the `.env` files in a workspace and reports keys missing from some files. Values are never read into comparison output, files are never modified, and there is no network access. Comparison runs automatically on file changes (debounced), on demand (`Show Issues`, `Compare Selected`), and against an optional template file.

## Architecture

```
extension.ts              activate(): build adapters -> createDetector -> registerAllCommands -> registerVSCodeWatchers -> initial checkSync
adapters/                 thin vscode wrappers, deps injected as typed bags:
  vscodeConfiguration     Configuration (get/has) over workspace.getConfiguration('envsync-le')
  vscodeFileSystem        findFiles/readFile/stat/asRelativePath
  vscodeNotifier          window messages, gated by notificationLevel
                          (all -> missing+extra+errors, important -> missing+errors, silent -> nothing)
  vscodeStatusBar         one status bar item; re-renders on config change
  vscodeTelemetry         local OutputChannel, only when telemetryEnabled
  vscodeUserInterface     quickPick (single + canPickMany), progress, messages
  vscodeWatcher           FileSystemWatcher per watchPattern; debounced checkSync;
                          rebuilt when watchPatterns changes
  vscodeCommand           CommandAdapter for openSettings
commands/                 one file per command; registerAllCommands wires the bag
detection/
  heuristics.ts           THE single place for basename/isEnvFileName/
                          detectFileType/shouldExcludeFile (glob matching)
  parser.ts               line-oriented key extraction; multiline quoted values
  comparator.ts           compareFiles(files, {mode, templatePath, caseSensitive})
                          -> SyncReport; extras only exist in template mode
  detector.ts             orchestrates discover -> parse -> compare -> notify/UI
config/config.ts          readConfig(configuration) snapshot; CONFIG_DEFAULTS table
utils/errors.ts           sanitizeErrorMessage/errorMessage — wired into every
                          user-facing error
types.ts                  shared types only — no logic
```

Conventions: factory functions + `Object.freeze` (no classes except the trivial `VSCodeCommandAdapter`), early returns, dependency bags typed inline at the consumer. Runtime strings are plain English; the 13 `package.nls*.json` catalogues localize **manifest** strings only (VS Code `%key%` substitution — do not add a runtime i18n layer without wiring real bundles).

## Invariants (things that were once broken — keep them true)

- **The bundle must be self-contained.** The VSIX ships `dist/extension.js` only; `scripts/check-bundle.js` (run in `vscode:prepublish` and CI) does a static require scan AND loads the bundle with `vscode` stubbed. Every v1.x VSIX failed activation because the tsc output required `vscode-nls` with no `node_modules` in the package.
- **`CONFIG_DEFAULTS` must equal package.json defaults.** `config.test.ts` asserts parity over every declared setting; add new settings to both plus the KEY_MAP in the test.
- **Every declared setting must have a consumer.** v1 shipped 17 no-op settings; don't add a setting without wiring it.
- **Detection behavior is pinned by golden snapshots** (`detection/characterization.test.ts` + `__fixtures__/`). Any output change must update goldens in the same commit and be listed in the CHANGELOG.
- **nls catalogues stay in key-parity:** all 12 locale files carry exactly the keys of `package.nls.json`.
- **Filename/path/glob heuristics live in one place** (`detection/heuristics.ts`). The watcher, parser, comparator, and commands must never re-implement basename splitting, env-file naming, or glob matching (v1 had three divergent copies).
- **Config is read per use, never snapshotted at construction.** The watcher and status bar re-read config on every event/render; the status bar and watcher also react to `onDidChangeConfiguration`.
- **`extraKeys` only exist in template mode.** Auto mode compares against the union of all keys, so nothing can be "extra" — don't fabricate extras there.

## Toolchain

- **Build:** esbuild bundle (`bun run build`, `build:prod` minified). `tsc` is typecheck-only (`noEmit`) and covers test files.
- **Unit tests:** vitest; `vscode` aliased to `src/__mocks__/vscode.ts` (stateful mock with `_reset/_set` helpers — config store, message log, command registry, file store, watcher registry). Coverage thresholds enforced: 80 lines / 80 funcs / 75 branches / 80 stmts.
- **Integration tests:** `bun run test:integration` — `@vscode/test-cli` launches a real VS Code against `test/integration/fixtures/` (committed `.env` + `.env.local` with a known missing key; config in `.vscode-test.mjs`, tests compiled via `tsconfig.it.json` to `out-test/`).
- **Lint/format:** Biome (tabs, single quotes). `__fixtures__`/`__snapshots__` are exempt — formatting fixtures would corrupt goldens.
- **Packaging:** `bun run package` → `release/*.vsix`. `.vscodeignore` is an allow-list; the VSIX is ~21 files.

## Release

1. Bump `version` in package.json, add a CHANGELOG entry.
2. CI green on all 3 OSes (includes packaging + integration tests).
3. `Release` workflow (manual dispatch) publishes to the VS Code Marketplace (`VSCE_PAT`) and Open VSX (`OVSX_PAT`) — Open VSX is what Cursor/VSCodium users install from. Locally: `bun run package` then `vsce publish` / `ovsx publish`.

## Known limitations (documented, not bugs)

- Keys must match `[A-Za-z_][A-Za-z0-9_-]*`; exotic keys (dots, unicode) are reported as parse errors rather than compared.
- An unterminated quoted value swallows the remainder of the file (dotenv itself behaves this way); it is reported as a parse error.
- Values are never parsed beyond quote tracking — inline `#` comments, escapes, and interpolation are irrelevant to key comparison and ignored.
- Glob support is `*`, `**`, `?` only — no braces, no character classes.
- `watchPatterns` are passed directly to `createFileSystemWatcher`; bare patterns like `.env*` watch workspace roots, `**/.env*` watches subdirectories (at more cost).
