# EnvSync-LE Testing

Comprehensive testing strategy ensuring reliability, performance, and production-grade sync detection quality.

## Testing Philosophy

- **Unit tests** validate pure functions (parsing, comparison) in isolation
- **Integration tests** prove resilience with real-world .env files
- **Contract tests** verify configuration side-effects and parse-error handling
- **Thin activation** isolates VS Code API from testable core logic
- **High coverage** maintained with fast feedback in CI

## Test Organization

```
src/
├── detection/
│   ├── parser.test.ts           # .env parsing and key extraction
│   ├── parser.integration.test.ts # Real .env fixtures
│   ├── comparator.test.ts       # File comparison logic
│   ├── detector.test.ts         # Sync orchestration
│   └── detector.integration.test.ts # End-to-end workflows
│   └── __data__/               # Test fixtures with expected outputs
│       ├── sample.env
│       ├── sample.env.expected.txt
│       ├── sample.env.local
│       ├── sample.env.example
│       └── invalid.env
├── config/
│   └── config.test.ts          # Configuration validation
├── adapters/
│   ├── vscodeConfiguration.test.ts
│   ├── vscodeFileSystem.test.ts
│   ├── vscodeNotifier.test.ts
│   └── vscodeWatcher.test.ts
└── commands/
    ├── compareSelected.test.ts
    ├── setTemplate.test.ts
    └── ignoreFile.test.ts
```

## Running Tests

```bash
# Full test suite
npm test

# Coverage report (text + HTML)
npm run test:coverage
# Output: coverage/index.html

# Watch mode for development
npm run test:watch

# Linting
npm run lint
npm run lint:fix
```

## Test Structure

### Unit Tests (Pure Functions)

```typescript
import { describe, expect, it } from 'vitest'
import { parseDotenvFile } from './parser'

describe('parseDotenvFile', () => {
  it('parses valid .env file', () => {
    const content = 'DATABASE_URL=postgresql://localhost:5432/db\nAPI_KEY=sk-12345'
    const result = parseDotenvFile(content, 'test.env')

    expect(result.success).toBe(true)
    expect(result.keys).toEqual(['DATABASE_URL', 'API_KEY'])
    expect(result.errors).toEqual([])
  })

  it('handles malformed lines gracefully', () => {
    const content = 'VALID_KEY=value\nINVALID_LINE\nANOTHER_KEY=value2'
    const result = parseDotenvFile(content, 'test.env')

    expect(result.success).toBe(true)
    expect(result.keys).toEqual(['VALID_KEY', 'ANOTHER_KEY'])
    expect(result.errors.length).toBeGreaterThan(0)
  })

  it('respects case sensitivity setting', () => {
    const content = 'DATABASE_URL=value\ndatabase_url=value2'

    const caseSensitive = parseDotenvFile(content, 'test.env', true)
    expect(caseSensitive.keys).toEqual(['DATABASE_URL', 'database_url'])

    const caseInsensitive = parseDotenvFile(content, 'test.env', false)
    expect(caseInsensitive.keys).toEqual(['DATABASE_URL'])
  })
})
```

### Integration Tests (Real Fixtures)

```typescript
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'fs'
import { join } from 'path'
import { parseDotenvFile } from './parser'

describe('parseDotenvFile integration', () => {
  const dataDir = join(__dirname, '__data__')

  it('parses sample.env correctly', () => {
    const content = readFileSync(join(dataDir, 'sample.env'), 'utf-8')
    const expected = readFileSync(join(dataDir, 'sample.env.expected.txt'), 'utf-8')
      .trim()
      .split('\n')
      .filter((line) => line.trim())

    const result = parseDotenvFile(content, 'sample.env')

    expect(result.success).toBe(true)
    expect(result.keys).toEqual(expected)
    expect(result.errors).toEqual([])
  })

  it('handles invalid.env with errors', () => {
    const content = readFileSync(join(dataDir, 'invalid.env'), 'utf-8')
    const result = parseDotenvFile(content, 'invalid.env')

    expect(result.success).toBe(true) // Graceful degradation
    expect(result.errors.length).toBeGreaterThan(0)
  })
})
```

### Command Tests

```typescript
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { Uri } from 'vscode'
import { registerCompareSelectedCommand } from './compareSelected'

describe('compareSelected command', () => {
  let detector: Detector
  let fileSystem: FileSystem
  let ui: UserInterface

  beforeEach(() => {
    vi.clearAllMocks()
    detector = { checkSyncForFiles: vi.fn().mockResolvedValue(undefined) }
    fileSystem = { asRelativePath: (p: string) => p }
    ui = { showWarningMessage: vi.fn() }
  })

  it('warns when fewer than two files are selected', async () => {
    const handler = createHandler({ detector, fileSystem, ui })
    await handler(Uri.file('/root/.env'))

    expect(ui.showWarningMessage).toHaveBeenCalledWith(expect.stringContaining('at least 2'))
  })

  it('triggers sync check for multiple files', async () => {
    const handler = createHandler({ detector, fileSystem, ui })
    const files = [Uri.file('/root/.env'), Uri.file('/root/.env.local')]

    await handler(files[0], files)

    expect(detector.checkSyncForFiles).toHaveBeenCalledWith(['/root/.env', '/root/.env.local'])
  })
})
```

## Fixture Conventions

### Fixture Layout

```
src/detection/__data__/
  sample.env                     # Base environment file
  sample.env.expected.txt        # Expected keys (sorted)
  sample.env.local               # Local overrides
  sample.env.local.expected.txt
  sample.env.example             # Template file
  sample.env.example.expected.txt
  invalid.env                    # Malformed file
  invalid.env.expected.txt       # Partial keys extracted
```

### Fixture Rules

- One comprehensive sample per file type + edge cases
- Expected files contain sorted, normalized keys (one per line)
- Invalid files test error handling and graceful degradation
- Cover common patterns: comments, quotes, multiline, empty lines

## Test Categories

### Core Logic Tests

- **Parser tests**: Key extraction, comment handling, case sensitivity, error recovery
- **Comparator tests**: Missing keys, extra keys, different comparison modes
- **Detector tests**: Sync orchestration, reporting, status calculation
- **Config tests**: Settings validation, defaults, frozen objects

### Adapter Tests

- **File system**: File discovery, pattern matching, reading
- **Watcher**: Debouncing, event handling, error recovery
- **Notifier**: Message levels, formatting, localization
- **Status bar**: Status updates, tooltips, click handling

### Integration Tests

- **End-to-end workflows**: Complete sync check with real files
- **Configuration integration**: Settings changes affecting behavior
- **Command integration**: User interactions and results

## Coverage Requirements

### Minimum Thresholds

- **Overall**: >90% line coverage
- **Core logic**: >95% for detection/ and config/
- **Commands**: >80% for commands/
- **Adapters**: >70% for adapters/ (thin wrappers)

### Coverage Exclusions

- Type definitions (`src/types.ts`)
- Test files (`*.test.ts`)
- Mock files (`src/__mocks__/`)
- VS Code API thin wrappers (single-line delegates)

### Coverage Reporting

```bash
npm run test:coverage
```

Output:

- **Text report**: Console summary with percentages
- **HTML report**: `coverage/index.html` for detailed analysis
- **LCOV report**: `coverage/lcov.info` for CI integration

## CI Integration

### GitHub Actions Pipeline

```yaml
- name: Run Tests
  run: npm test

- name: Generate Coverage
  run: npm run test:coverage

- name: Upload Coverage
  uses: codecov/codecov-action@v3
  with:
    file: ./coverage/lcov.info
```

Requirements:

- All tests pass (zero failures)
- Coverage >90% overall
- No linting errors
- TypeScript compilation succeeds

## Performance Testing

### Benchmark Tests

```typescript
describe('Performance benchmarks', () => {
  it('completes sync check in <100ms for small workspace', async () => {
    const startTime = Date.now()
    await detector.checkSync()
    const duration = Date.now() - startTime

    expect(duration).toBeLessThan(100)
  })

  it('handles large workspace efficiently', async () => {
    const largeWorkspace = createLargeWorkspace(1000) // 1000 .env files
    const startTime = Date.now()

    await detector.checkSync()

    const duration = Date.now() - startTime
    expect(duration).toBeLessThan(5000) // 5 seconds
  })

  it('prevents memory leaks during repeated checks', async () => {
    const initialMemory = process.memoryUsage().heapUsed

    for (let i = 0; i < 100; i++) {
      await detector.checkSync()
    }

    if (global.gc) global.gc()

    const finalMemory = process.memoryUsage().heapUsed
    const increase = finalMemory - initialMemory

    expect(increase).toBeLessThan(10 * 1024 * 1024) // 10MB
  })
})
```

## Debugging Tests

### Common Issues

- **Mock not working**: Check `vi.clearAllMocks()` in `beforeEach`
- **Async timing**: Use proper `async/await`, avoid `.then()` chains
- **File paths**: Use `join(__dirname, '__data__')` for fixtures
- **Frozen objects**: Cannot modify test data; create new objects

### Debug Strategies

- Use `console.log` sparingly for debugging
- Set breakpoints in VS Code test debugger
- Verify mock call counts: `expect(mock).toHaveBeenCalledTimes(n)`
- Isolate failing tests with `.only`

## Best Practices

### Test Organization

- Group related tests in `describe` blocks
- Use descriptive test names that explain expected behavior
- One assertion per test when possible
- Follow Arrange-Act-Assert pattern

### Test Data

- Use realistic fixtures from production .env files
- Include edge cases (comments, quotes, empty lines, duplicates)
- Keep test data minimal but representative
- Mock external dependencies (file system, VS Code APIs)

### Performance

- Keep tests fast (<5s total suite time)
- Use Vitest parallel execution (default)
- Mock slow operations (file I/O, network)
- Avoid unnecessary setup/teardown

### Maintenance

- Update fixtures when parser logic changes
- Review coverage reports regularly
- Refactor tests alongside code
- Document complex test scenarios

---

**Project:** [Issues](https://github.com/nolindnaidoo/envsync-le/issues) • [Pull Requests](https://github.com/nolindnaidoo/envsync-le/pulls)

**Docs:** [Architecture](ARCHITECTURE.md) • [Testing](TESTING.md) • [Performance](PERFORMANCE.md) • [Commands](COMMANDS.md) • [Configuration](CONFIGURATION.md) • [Development](DEVELOPMENT.md) • [Troubleshooting](TROUBLESHOOTING.md)
