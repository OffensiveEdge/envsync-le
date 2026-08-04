<p align="center">
  <img src="src/assets/images/icon.png" alt="EnvSync-LE Logo" width="96" height="96"/>
</p>
<h1 align="center">EnvSync-LE: Zero Hassle .env Sync</h1>
<p align="center">
  <b>Spot missing keys across your .env files before they bite</b><br/>
  <i>Automatic checks, a status bar counter, and a plain markdown report</i>
</p>

<p align="center">
  <a href="https://marketplace.visualstudio.com/items?itemName=nolindnaidoo.envsync-le">
    <img src="https://img.shields.io/badge/Install%20from-VS%20Code-blue?style=for-the-badge&logo=visualstudiocode" alt="Install from VS Code Marketplace" />
  </a>
  <a href="https://letools.dev">
    <img src="https://img.shields.io/badge/LE%20Tools-letools.dev-blue?style=for-the-badge" alt="LE Tools" />
  </a>
</p>

---

<p align="center">
  <img src="src/assets/images/demo.gif" alt="EnvSync-LE Demo" style="max-width: 100%; height: auto;" />
</p>

## What it does

EnvSync-LE compares the **variable names** across the `.env` files in your
workspace and tells you which files are missing which keys. It never reads
or compares values, and it never modifies your files. Works in VS Code and
in VS Code–based editors like Cursor and VSCodium (installable from Open VSX).

- **Automatic checks** — a debounced sync check runs whenever a watched `.env` file changes
- **Status bar counter** — the current issue count, one click away from the full report
- **Markdown report** — `Show Details` (`Ctrl+Alt+S` / `Cmd+Alt+S`) lists every missing key per file
- **Three comparison modes** — `auto` (union of all keys), `manual` (only the files you list), `template` (validate everything against one reference file; also reports keys a file has that the template lacks)
- **Ignore list** — temporarily exclude files (e.g. `.env.example`) from checking

## File classification

| Filename | Type |
|---|---|
| `.env` | base |
| `.env.local`, `.env.*.local` | local override |
| `.env.example`, `.env.template` | template |
| `.env.production`, `.env.prod` | production |
| `.env.development`, `.env.dev` | development |
| `.env.test` | test |
| `anything.env`, unknown segments | base |

Classification is segment-based on the basename — `app.device.env` is not
"development", and Windows paths classify correctly.

## Commands

| Command | Description |
|---|---|
| `EnvSync-LE: Show Details` (`Ctrl+Alt+S` / `Cmd+Alt+S`) | Run a sync check and open the markdown report |
| `EnvSync-LE: Compare Files` | Compare the key sets of two or more selected `.env` files |
| `EnvSync-LE: Set Template` | Mark a `.env` file as the reference template |
| `EnvSync-LE: Clear Template` | Return to automatic comparison |
| `EnvSync-LE: Ignore File` | Exclude a `.env` file from sync checking |
| `EnvSync-LE: Unignore File` | Re-include a previously ignored file |
| `EnvSync-LE: Clear Ignored Files` | Empty the ignore list |
| `EnvSync-LE: Open Settings` | Open VS Code settings filtered to EnvSync-LE |
| `EnvSync-LE: Help & Troubleshooting` | Built-in documentation |

## Settings

| Setting | Default | Description |
|---|---|---|
| `envsync-le.enabled` | `true` | Master switch for sync checking |
| `envsync-le.watchPatterns` | `[".env*"]` | Glob patterns for files to watch and compare |
| `envsync-le.excludePatterns` | `[]` | Glob patterns to exclude; patterns without `/` match the basename (so `.env.*.local` excludes nested files too) |
| `envsync-le.notificationLevel` | `important` | `all` = missing + extra keys, `important` = missing keys + errors, `silent` = nothing |
| `envsync-le.statusBar.enabled` | `true` | Show the status bar item (applies immediately) |
| `envsync-le.debounceMs` | `1000` | Delay between a file change and the sync check |
| `envsync-le.caseSensitive` | `true` | Treat `DB_URL` and `db_url` as different keys |
| `envsync-le.comparisonMode` | `auto` | `auto`, `manual`, or `template` |
| `envsync-le.compareOnlyFiles` | `[]` | In manual mode, only compare these files (workspace-relative) |
| `envsync-le.templateFile` | — | The reference file for template mode (set via `Set Template`) |
| `envsync-le.temporaryIgnore` | `[]` | Files currently ignored (managed by the ignore commands) |
| `envsync-le.telemetryEnabled` | `false` | Local-only event log (see Privacy) |

The settings UI is translated into 12 languages besides English.

## Parsing notes

- Keys must match `[A-Za-z_][A-Za-z0-9_-]*`; anything else left of `=` is
  reported as a parse error for that line (the rest of the file still counts).
- `export KEY=value` is accepted; duplicate keys count once.
- Values quoted with `"`, `'`, or `` ` `` may span multiple lines; an
  unterminated quote is reported as an error.
- In `auto` mode the reference is the union of all keys, so "extra keys"
  only exist in `template` mode.

## Privacy & security

- **No network access.** The extension never sends data anywhere. The
  `telemetryEnabled` setting only writes events to a local Output Channel
  you can inspect (`envsync-le`).
- **Values are never read, displayed, or logged** — only key names are compared.
- Error notifications redact home directories and credential-shaped fragments.

## Development

```bash
bun install
bun run build            # esbuild bundle -> dist/extension.js
bun run typecheck        # tsc --noEmit (includes tests)
bun run test             # vitest unit suite
bun run test:integration # real VS Code extension host
bun run lint             # biome
bun run package          # VSIX into release/
```

Architecture and conventions live in [AGENTS.md](AGENTS.md). Changes are tracked in [CHANGELOG.md](CHANGELOG.md).

## Performance

<!-- performance:start -->
| Input | Size | Found | Time | Rate | Scan speed |
| --- | --- | --- | --- | --- | --- |
| 4 files x 500 keys | 0.02 MB | 1,910 | 0.18 ms | 10,816,443/sec | 102.3 MB/s |
| 24 files x 500 keys | 0.11 MB | 11,103 | 0.84 ms | 13,263,252/sec | 125.5 MB/s |
| 4 files x 8,000 keys | 0.32 MB | 31,172 | 1.48 ms | 21,071,644/sec | 218.4 MB/s |

Median of 7 runs after warmup, on Apple M5 Pro, 24 GB RAM, Node 24.3.0. Inputs are generated
by `scripts/benchmark.ts` rather than checked in, so the sizes above are
exactly what was measured. Reproduce with `bun run benchmark`.

These are machine-specific and are not asserted in CI — a benchmark that gates
a build only tells you how busy the runner was.
<!-- performance:end -->

## Testing

<!-- coverage:start -->
| Metric | Coverage |
| --- | --- |
| Statements | 82.27% |
| Branches | 73.27% |
| Functions | 87.55% |
| Lines | 83.04% |

115 test cases across 10 files, plus an integration suite that runs
in a real VS Code extension host and an end-to-end test that installs the
built `.vsix` into a clean profile.

Generated from `coverage/coverage-summary.json` by
`scripts/coverage-readme.js`; CI fails if this section drifts from a fresh
run. Reproduce with `bun run test:coverage`.
<!-- coverage:end -->

## More from the LE Family

Every tool in the family, one page: **[letools.dev](https://letools.dev)**

- **[String-LE](https://marketplace.visualstudio.com/items?itemName=nolindnaidoo.string-le)** - Extract string values for i18n from JSON, YAML, CSV, TOML, INI, and .env
- **[Numbers-LE](https://marketplace.visualstudio.com/items?itemName=nolindnaidoo.numbers-le)** - Extract numeric values from JSON, YAML, CSV, TOML, INI, and .env
- **[Paths-LE](https://marketplace.visualstudio.com/items?itemName=nolindnaidoo.paths-le)** - Extract file paths from JS/TS imports, JSON, HTML, CSS, TOML, CSV, and .env
- **[Regex-LE](https://marketplace.visualstudio.com/items?itemName=nolindnaidoo.regex-le)** - Find, test, and validate regular expressions with ReDoS screening
- **[Secrets-LE](https://marketplace.visualstudio.com/items?itemName=nolindnaidoo.secrets-le)** - Detect and sanitize credentials locally, before you commit
- **[Scrape-LE](https://marketplace.visualstudio.com/items?itemName=nolindnaidoo.scrape-le)** - Check whether a page is scrapeable before you write the scraper
- **[Colors-LE](https://marketplace.visualstudio.com/items?itemName=nolindnaidoo.colors-le)** - Extract and analyze colors from CSS, SCSS, LESS, Stylus, HTML, JS/TS, and SVG
- **[URLs-LE](https://marketplace.visualstudio.com/items?itemName=nolindnaidoo.urls-le)** - Extract URLs from documentation, configs, and code
- **[Dates-LE](https://marketplace.visualstudio.com/items?itemName=nolindnaidoo.dates-le)** - Extract and analyze dates from logs, configs, and code

## License

MIT © [nolindnaidoo](https://github.com/nolindnaidoo)
