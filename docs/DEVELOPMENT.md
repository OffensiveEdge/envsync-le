# EnvSync-LE Development Guide

This guide helps you set up a local development environment, run and debug the extension, and contribute with confidence. It complements `CONTRIBUTING.md` and the deeper `SPECIFICATION.md`.

## Prerequisites

- Node.js 20+
- VS Code 1.70+
- Git
- npm or yarn

## Setup

```bash
git clone https://github.com/nolindnaidoo/envsync-le.git
cd envsync-le
npm ci
```

## Everyday Commands

```bash
npm test               # run tests (Vitest)
npm run test:coverage  # coverage (text + HTML)
npm run lint           # Biome check
npm run lint:fix       # Biome auto-fix
npm run build          # type-check + compile to dist/
npm run watch          # tsc --watch
npm run package        # create VSIX package
```

## Run & Debug

1. Open the folder in VS Code
2. Press F5 (Run and Debug → "Launch Extension")
3. A new Extension Development Host opens with EnvSync-LE installed

**Tips**:

- Set breakpoints across `src/**` and use the Debug Console
- Use the Status Bar entry and Command Palette inside the dev host
- Check the Output channel for "EnvSync-LE" logs

## Project Structure (Overview)

```
src/
  extension.ts         # activation: register commands/watchers only
  adapters/           # VS Code API abstractions
  commands/           # compare, set template, ignore, show issues
  config/             # frozen settings model and validation
  detection/          # core business logic (parser, comparator, detector)
  interfaces/         # type definitions for adapters
  types.ts            # core type definitions
  __data__/           # test fixtures and expected outputs
```

See `ARCHITECTURE.md` for detailed architecture information.

## Code Style and Patterns

- **Functional Programming**: Prefer factory functions over classes
- **Strict TypeScript**: Use `readonly` types and `Object.freeze()` for immutability
- **Keep `src/extension.ts` minimal**: Only register commands/providers
- **Centralize types**: Use `src/types.ts` for core types
- **Dependency Injection**: Inject dependencies via factory functions

Refer to `CONTRIBUTING.md` for detailed style rules and patterns.

## Testing

### Test Structure

- **Unit Tests**: Pure functions and core logic (`src/**/*.test.ts`)
- **Integration Tests**: End-to-end workflows (`src/**/*.integration.test.ts`)
- **Fixtures**: Test data in `src/detection/__data__/`

### Running Tests

```bash
npm test                    # run all tests
npm run test:coverage      # run with coverage
npm run test:watch         # watch mode
```

### Test Data

```
src/detection/__data__/
  sample.env              # input
  sample.env.expected.txt # expected keys
  sample.env.local        # input
  sample.env.local.expected.txt # expected keys
  invalid.env             # malformed input
  invalid.env.expected.txt # expected keys (with errors)
```

### Writing Tests

```typescript
import { describe, expect, it } from 'vitest'
import { parseDotenvFile } from './parser'

describe('parseDotenvFile', () => {
  it('should parse valid .env file', () => {
    const content = 'DATABASE_URL=postgresql://localhost:5432/db\nAPI_KEY=sk-12345'
    const result = parseDotenvFile(content, 'test.env')

    expect(result.success).toBe(true)
    expect(result.keys).toEqual(['DATABASE_URL', 'API_KEY'])
    expect(result.errors).toEqual([])
  })
})
```

## Architecture Patterns

### Factory Pattern

```typescript
export function createDetector(deps: DetectorDependencies): Detector {
  return Object.freeze({
    checkSync: async () => {
      /* implementation */
    },
    dispose: () => {
      /* cleanup */
    },
  })
}
```

### Dependency Injection

```typescript
export function registerCommands(
  context: vscode.ExtensionContext,
  deps: Readonly<{
    telemetry: Telemetry
    notifier: Notifier
    detector: Detector
    // ... other dependencies
  }>,
): void {
  // Register commands with injected dependencies
}
```

### Immutable Data

```typescript
export interface SyncReport {
  readonly status: SyncStatus
  readonly files: readonly DotenvFile[]
  readonly missingKeys: readonly MissingKey[]
  readonly extraKeys: readonly ExtraKey[]
  readonly errors: readonly ParseError[]
  readonly lastChecked: number
}
```

## Debugging

### VS Code Extension Development

1. **Set Breakpoints**: In `src/**` files
2. **Debug Console**: Use for inspection and evaluation
3. **Output Channel**: Check "EnvSync-LE" output for logs
4. **Extension Host**: Use the Extension Development Host for testing

### Common Debug Scenarios

- **File Detection**: Check if files are being discovered
- **Parsing Issues**: Verify .env file parsing
- **Sync Logic**: Debug comparison and sync detection
- **Configuration**: Verify settings are being read correctly

### Debug Configuration

```json
{
  "type": "extensionHost",
  "request": "launch",
  "name": "Launch Extension",
  "runtimeExecutable": "${execPath}",
  "args": ["--extensionDevelopmentPath=${workspaceFolder}"],
  "outFiles": ["${workspaceFolder}/dist/**/*.js"],
  "preLaunchTask": "npm: build"
}
```

## Building and Packaging

### Development Build

```bash
npm run build    # TypeScript compilation
npm run watch    # Watch mode for development
```

### Production Build

```bash
npm run package  # Create VSIX package
```

### VSIX Package

The package command creates a `.vsix` file in the `release/` directory:

```
release/envsync-le-1.1.4.vsix
```

## Configuration Development

### Adding New Settings

1. **Update Types**: Add to `src/types.ts`
2. **Update Config**: Add to `src/config/config.ts`
3. **Update Schema**: Add to `package.json` contributes.configuration
4. **Update Tests**: Add tests for new settings
5. **Update Docs**: Update `CONFIGURATION.md`

### Configuration Pattern

```typescript
export function readConfig(configuration: Configuration): DotSyncConfig {
  const enabled = Boolean(configuration.get('enabled', true))
  const watchPatterns = configuration.get('watchPatterns', ['**/.env*']) as string[]

  return Object.freeze({
    enabled,
    watchPatterns: Object.freeze([...watchPatterns]),
    // ... other settings
  })
}
```

## Command Development

### Adding New Commands

1. **Create Command**: Add to `src/commands/`
2. **Register Command**: Add to `src/commands/index.ts`
3. **Update Manifest**: Add to `package.json` contributes.commands
4. **Add Tests**: Create tests for the command
5. **Update Docs**: Update `COMMANDS.md`

### Command Pattern

```typescript
export function registerNewCommand(
  context: vscode.ExtensionContext,
  deps: Readonly<{
    telemetry: Telemetry
    notifier: Notifier
    // ... other dependencies
  }>,
): void {
  const disposable = vscode.commands.registerCommand('envsync-le.newCommand', async () => {
    deps.telemetry.event('command', { name: 'newCommand' })

    // Command implementation
    deps.notifier.showInfo('Command executed successfully')
  })

  context.subscriptions.push(disposable)
}
```

## File System Integration

### File Watching

```typescript
export function registerVSCodeWatchers(
  context: vscode.ExtensionContext,
  detector: Detector,
  configuration: Configuration,
  fileSystem: FileSystem,
): void {
  const watcher = vscode.workspace.createFileSystemWatcher('**/.env*')

  watcher.onDidChange(
    debounce((uri) => {
      detector.checkSync()
    }, debounceMs),
  )

  context.subscriptions.push(watcher)
}
```

### File Discovery

```typescript
export async function discoverDotenvFiles(
  fileSystem: FileSystem,
  watchPatterns: readonly string[],
  excludePatterns: readonly string[],
  config: DotSyncConfig,
): Promise<{ files: DotenvFile[]; errors: ParseError[] }> {
  // Implementation
}
```

## Error Handling

### Error Types

```typescript
export interface ParseError {
  readonly filepath: string
  readonly message: string
  readonly line?: number
  readonly column?: number
}
```

### Error Handling Pattern

```typescript
export function parseDotenvFile(content: string, filepath: string): ParseResult {
  try {
    const keys = extractKeys(content)
    return {
      success: true,
      keys: Object.freeze(keys),
      errors: Object.freeze([]),
    }
  } catch (error) {
    return {
      success: false,
      keys: Object.freeze([]),
      errors: Object.freeze([
        {
          filepath,
          message: error.message,
        },
      ]),
    }
  }
}
```

## Performance Considerations

### File Watching

- **Debouncing**: Prevent excessive sync checks
- **Pattern Matching**: Efficient glob pattern matching
- **Exclusion**: Skip excluded files early
- **Batching**: Group related file changes

### Memory Management

- **Immutable Data**: Prevent accidental mutations
- **Lazy Loading**: Load files only when needed
- **Cleanup**: Dispose of resources properly
- **Caching**: Cache parsed results when appropriate

### User Experience

- **Progress Indicators**: Show progress for long operations
- **Cancellation**: Support operation cancellation
- **Background Processing**: Non-blocking operations
- **Status Updates**: Real-time status bar updates

## Localization

### Adding New Strings

1. **Add to package.nls.json**: Base English strings
2. **Add to language.json**: Source catalogue
3. **Update translations**: Add to locale-specific files
4. **Use in code**: Use `localize()` function

### Localization Pattern

```typescript
import * as nls from 'vscode-nls'
const localize = nls.config({ messageFormat: nls.MessageFormat.file })()

const message = localize('runtime.message.no-env-files', 'No .env files found in workspace')
```

## Contributing

### Code Style

- **Biome**: Use Biome for linting and formatting
- **TypeScript**: Strict mode with explicit types
- **Functional**: Prefer functional programming patterns
- **Immutable**: Use `readonly` and `Object.freeze()`

### Commit Messages

- **Format**: `type(scope): description`
- **Types**: feat, fix, docs, style, refactor, test, chore
- **Scope**: commands, config, detection, adapters, etc.

### Pull Requests

1. **Fork**: Fork the repository
2. **Branch**: Create feature branch
3. **Develop**: Implement changes with tests
4. **Test**: Run full test suite
5. **Document**: Update documentation
6. **Submit**: Create pull request

### Testing Requirements

- **Coverage**: Maintain >90% test coverage
- **Unit Tests**: Test pure functions in isolation
- **Integration Tests**: Test end-to-end workflows
- **Edge Cases**: Test error conditions and edge cases

## Troubleshooting Development

### Common Issues

- **Build Errors**: Check TypeScript errors, run `npm run build`
- **Test Failures**: Check test output, verify fixtures
- **Extension Not Loading**: Check activation events, reload window
- **Commands Not Working**: Check command registration, verify dependencies

### Debug Tips

- **Use Output Channel**: Check "EnvSync-LE" output for logs
- **Set Breakpoints**: Use VS Code debugger effectively
- **Check Console**: Use Debug Console for evaluation
- **Verify Configuration**: Check settings are being read correctly

### Performance Debugging

- **Profile Memory**: Use VS Code memory profiler
- **Check File Operations**: Monitor file system operations
- **Verify Debouncing**: Check debounce is working correctly
- **Monitor CPU**: Check CPU usage during operations

This development guide provides comprehensive information for contributing to EnvSync-LE and maintaining high code quality standards.
