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

Conventions: factory functions + `Object.freeze` (no classes except the trivial `VSCodeCommandAdapter`), early returns, dependency bags typed inline at the consumer. Runtime strings are plain English; the `package.nls*.json` catalogues localize **manifest** strings only (VS Code `%key%` substitution — do not add a runtime i18n layer without wiring real bundles).

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

- **Runtime targets:** `engines.vscode` is the supported floor and `@types/vscode` is pinned to it **exactly**. A caret there lets the type surface drift ahead of the version users actually run, so code compiles against APIs that are not there at runtime. Dependabot is configured to never bump it.
- **Build:** esbuild bundle (`bun run build`, `build:prod` minified). `tsc` is typecheck-only (`noEmit`) and covers test files. TypeScript 7.
- **Unit tests:** vitest 4; `vscode` aliased to `src/__mocks__/vscode.ts` (stateful mock with `_reset/_set` helpers). Coverage provider `v8`, thresholds enforced at **75 lines / 80 functions / 60 branches / 75 statements**. These are a floor to ratchet upward, never to lower so a build passes.
- **Integration tests:** `bun run test:integration` — `@vscode/test-cli` launches a real VS Code (config in `.vscode-test.mjs`, tests compiled via `tsconfig.it.json` to `out-test/`). That project targets `node16` module resolution; TypeScript 7 removed `node10`, which `"Node"` resolved to.
- **Installed-VSIX tests:** `bun run test:e2e-vsix` installs the built `.vsix` into a clean VS Code profile and drives it. This is the only test that exercises the artifact users receive, and it runs in CI.
- **Lint/format:** Biome (tabs, single quotes). `__fixtures__`/`__snapshots__` are exempt — formatting fixtures would corrupt goldens. `biome.json` is byte-identical across all ten repos; change it in one and copy it to the rest.
- **Packaging:** `bun run package` → `release/*.vsix`. `.vscodeignore` is an allow-list; the VSIX is ~21 files. Packaging uses `--no-dependencies`: the bundle is self-contained, so walking the npm tree served no purpose and broke after any dependency change.
- **Localization:** The 12 `package.nls.*.json` catalogues in `src/i18n/` localize **manifest** strings only (VS Code `%key%` substitution) and are copied to the package root at prepublish.

## Generated documentation

Two README sections are generated. Do not hand-edit the content between their markers.

- `bun run test:coverage && bun run coverage:readme` writes the Testing section from `coverage/coverage-summary.json`. CI runs `coverage:readme:check`, which fails when the committed numbers no longer match a real run — coverage is compared within 1 percentage point (it is not bit-identical across machines), while test counts are derived from source and must match exactly.
- `bun run benchmark && bun run perf:readme` writes the Performance section from a real run of the extraction entry point. This is **not** checked in CI: throughput is machine-specific, so a hosted runner would fail it for reasons that say nothing about the code. The host is printed with the numbers instead.

The pre-2.0 README carried hand-written test counts and throughput figures that drifted until they were false. Generating them is what stops that recurring.

## Security & automation

- **CodeQL** runs on push, PR and weekly (`javascript-typescript` + `actions`), configured in `.github/codeql-config.yml`. Test files and fixtures are excluded on purpose: they contain inputs that are supposed to look dangerous, and scanning them produces findings that can only ever be dismissed.
- **Dependabot** (`bun` ecosystem, not `npm` — the npm updater rewrites `package.json` without regenerating `bun.lock`, so its PRs can never pass the frozen-lockfile gate) opens grouped weekly PRs.
- **Auto-merge** is workflow-driven, not GitHub-native: `main` has no required status checks, so native auto-merge would land a PR before CI started. `dependabot-auto-merge.yml` waits for the CI run to conclude and merges only patch/minor **devDependency** updates. Runtime dependencies bundle into the shipped VSIX and always need a human.
- **Actions are pinned to commit SHAs.** A tag is mutable and this repo holds a publish token. The trailing `# vX.Y.Z` comment is what Dependabot reads and rewrites.
- **Branch safety:** a `main-safety` ruleset blocks deletion and force-push. Pushes to `main` are otherwise unrestricted by design.
- Secret scanning and push protection are enabled. `VSCE_PAT` and `OVSX_PAT` live in repo secrets and in Doppler (`extensions` / `prd`).

## Release

1. Bump `version` in package.json and write the CHANGELOG entry. The entry must describe what actually changed, including bug fixes — it ships inside the VSIX and renders on the listing page.
2. Regenerate the README sections (`coverage:readme`, and `perf:readme` if behaviour changed) and commit them.
3. CI green on all three OSes. That includes lint, typecheck, coverage, the bundle gate, packaging, integration tests, and the installed-VSIX e2e.
4. Tag the commit being released, so the tag is the artifact rather than an approximation of it.
5. Dispatch the `Release` workflow. It takes two independent opt-ins — `marketplace` (default **on**) and `openvsx` (default **off**) — because a version cannot be republished, so a run that publishes one registry and fails on the other is only recoverable by re-running with the failed target alone. It validates credentials before doing anything irreversible.

**Open VSX defaults off deliberately.** `ovsx publish` takes no namespace argument; it derives the namespace from `publisher` in the VSIX. Enabling it publishes to whatever `package.json` currently names, with no confirmation.

## Known limitations (documented, not bugs)

- Keys must match `[A-Za-z_][A-Za-z0-9_-]*`; exotic keys (dots, unicode) are reported as parse errors rather than compared.
- An unterminated quoted value swallows the remainder of the file (dotenv itself behaves this way); it is reported as a parse error.
- Values are never parsed beyond quote tracking — inline `#` comments, escapes, and interpolation are irrelevant to key comparison and ignored.
- Glob support is `*`, `**`, `?` only — no braces, no character classes.
- `watchPatterns` are passed directly to `createFileSystemWatcher`; bare patterns like `.env*` watch workspace roots, `**/.env*` watches subdirectories (at more cost).
