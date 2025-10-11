# EnvSync-LE Technical Specification

This document defines the technical requirements, behavior, and implementation details for the EnvSync-LE VS Code extension.

## Overview

EnvSync-LE is a VS Code extension that detects, compares, and synchronizes .env files across workspaces. It provides real-time monitoring, visual diff capabilities, and automated sync checking to maintain consistency across environment configurations.

## Design Goals

- **Zero Configuration**: Works out of the box with sensible defaults
- **Real-time Monitoring**: Automatic detection of .env file changes
- **Visual Feedback**: Clear status indicators and diff visualization
- **Flexible Comparison**: Support for different comparison modes
- **Performance**: Efficient file watching and minimal resource usage
- **Privacy**: Local-only processing with no external dependencies

## Functional Requirements

### Core Features

#### 1. File Discovery

- **Requirement**: Automatically discover .env files in workspace
- **Patterns**: Support configurable file patterns (default: `**/.env*`)
- **Exclusions**: Support exclusion patterns (default: `.env.*.local`)
- **File Types**: Recognize different .env file types:
  - Base files (`.env`)
  - Local overrides (`.env.local`, `.env.development.local`)
  - Examples (`.env.example`, `.env.template`)
  - Environment-specific (`.env.production`, `.env.development`, `.env.test`)

#### 2. Sync Detection

- **Requirement**: Detect synchronization status between .env files
- **Comparison Modes**:
  - **Auto**: Compare all files against each other
  - **Template**: Compare files against a designated template
  - **Manual**: Compare selected files only
- **Status Types**:
  - `in-sync`: All files have consistent keys
  - `missing-keys`: Some files are missing keys present in others
  - `extra-keys`: Some files have keys not present in others
  - `no-files`: No .env files found in workspace

#### 3. Real-time Monitoring

- **Requirement**: Monitor file changes and update sync status
- **Debouncing**: Prevent excessive checks with configurable debounce (default: 1000ms)
- **File Watching**: Use VS Code file system watchers
- **Performance**: Efficient pattern matching and exclusion

#### 4. User Interface

- **Status Bar**: Show sync status with click-to-check functionality
- **Notifications**: Configurable notification levels (all, important, silent)
- **Commands**: Provide commands for manual sync checking and file comparison
- **Context Menus**: File-specific actions in explorer

#### 5. Error Handling

- **Parse Errors**: Handle malformed .env files gracefully
- **File System Errors**: Handle missing files and permission issues
- **User Feedback**: Provide clear error messages and suggestions
- **Recovery**: Automatic retry for transient errors

### Configuration

#### Settings Schema

```json
{
  "envsync-le.enabled": {
    "type": "boolean",
    "default": true,
    "description": "Enable EnvSync-LE extension"
  },
  "envsync-le.watchPatterns": {
    "type": "array",
    "default": ["**/.env*"],
    "description": "File patterns to watch for .env files"
  },
  "envsync-le.excludePatterns": {
    "type": "array",
    "default": [".env.*.local"],
    "description": "File patterns to exclude from watching"
  },
  "envsync-le.notificationLevel": {
    "type": "string",
    "enum": ["all", "important", "silent"],
    "default": "important",
    "description": "Notification verbosity level"
  },
  "envsync-le.statusBar.enabled": {
    "type": "boolean",
    "default": true,
    "description": "Show status bar item"
  },
  "envsync-le.debounceMs": {
    "type": "number",
    "minimum": 100,
    "default": 1000,
    "description": "Debounce delay for file change events"
  },
  "envsync-le.ignoreComments": {
    "type": "boolean",
    "default": true,
    "description": "Ignore comment lines in .env files"
  },
  "envsync-le.caseSensitive": {
    "type": "boolean",
    "default": true,
    "description": "Case-sensitive key comparison"
  },
  "envsync-le.comparisonMode": {
    "type": "string",
    "enum": ["auto", "manual", "template"],
    "default": "auto",
    "description": "Comparison mode for sync detection"
  },
  "envsync-le.templateFile": {
    "type": "string",
    "description": "Template file for template comparison mode"
  }
}
```

### Commands

#### 1. Show Issues (`envsync-le.showIssues`)

- **Purpose**: Display current sync status and issues
- **Availability**: Always available
- **Output**: Markdown report with sync status and missing/extra keys
- **Side Effects**: None

#### 2. Compare Selected (`envsync-le.compareSelected`)

- **Purpose**: Compare selected .env files
- **Availability**: When .env files are selected in explorer
- **Input**: Selected file URIs
- **Output**: Comparison report
- **Side Effects**: None

#### 3. Set Template (`envsync-le.setTemplate`)

- **Purpose**: Set template file for comparison
- **Availability**: When .env files are present
- **Input**: Template file selection
- **Output**: Confirmation message
- **Side Effects**: Updates configuration

#### 4. Ignore File (`envsync-le.ignoreFile`)

- **Purpose**: Temporarily ignore a file from sync checking
- **Availability**: When .env files are selected
- **Input**: File selection
- **Output**: Confirmation message
- **Side Effects**: Updates temporary ignore list

### File Parsing

#### .env File Format

- **Syntax**: `KEY=VALUE` pairs, one per line
- **Comments**: Lines starting with `#` (ignored if `ignoreComments` is true)
- **Empty Lines**: Ignored
- **Quotes**: Support for quoted values with escape sequences
- **Multiline**: Support for multiline values with `\n` escape

#### Parsing Rules

- **Key Extraction**: Extract keys from valid `KEY=VALUE` lines
- **Error Handling**: Skip malformed lines, log parse errors
- **Case Sensitivity**: Respect `caseSensitive` setting
- **Comments**: Ignore comment lines if configured
- **Whitespace**: Trim keys and values

#### Supported File Types

- **Base**: `.env` - Main environment file
- **Local**: `.env.local` - Local overrides (typically gitignored)
- **Development**: `.env.development`, `.env.dev` - Development environment
- **Production**: `.env.production`, `.env.prod` - Production environment
- **Test**: `.env.test` - Test environment
- **Example**: `.env.example`, `.env.template` - Template/example file

### Comparison Logic

#### Auto Mode

- **Reference**: Use the file with the most keys as reference
- **Comparison**: Compare all other files against the reference
- **Missing Keys**: Report keys present in reference but missing in other files
- **Extra Keys**: Report keys present in other files but missing in reference

#### Template Mode

- **Reference**: Use designated template file as reference
- **Comparison**: Compare all other files against the template
- **Missing Keys**: Report keys present in template but missing in other files
- **Extra Keys**: Not reported (template is authoritative)

#### Manual Mode

- **Reference**: User-selected files
- **Comparison**: Compare selected files only
- **Missing Keys**: Report keys present in one file but missing in others
- **Extra Keys**: Report keys present in one file but missing in others

### Performance Requirements

#### File Watching

- **Debouncing**: Minimum 100ms debounce delay
- **Pattern Matching**: Efficient glob pattern matching
- **Exclusion**: Early exclusion of ignored files
- **Memory**: Minimal memory usage for file watchers

#### Sync Checking

- **Response Time**: < 100ms for small workspaces (< 10 files)
- **Memory Usage**: < 10MB for typical workspace
- **CPU Usage**: < 5% during sync check
- **File I/O**: Minimize file system operations

#### User Interface

- **Status Updates**: Real-time status bar updates
- **Notifications**: Non-blocking notifications
- **Progress**: Progress indicators for long operations
- **Cancellation**: Support for operation cancellation

### Error Handling

#### Parse Errors

- **Malformed Lines**: Skip malformed lines, continue parsing
- **File Encoding**: Handle different file encodings gracefully
- **Large Files**: Handle large files without memory issues
- **User Feedback**: Show parse errors in notifications (if enabled)

#### File System Errors

- **Missing Files**: Handle deleted files gracefully
- **Permission Errors**: Show appropriate error messages
- **Network Drives**: Handle network drive issues
- **Recovery**: Automatic retry for transient errors

#### Configuration Errors

- **Invalid Settings**: Use defaults for invalid settings
- **Missing Settings**: Use sensible defaults
- **Migration**: Handle configuration migration gracefully
- **Validation**: Validate settings on change

### Security Considerations

#### File Access

- **Workspace Trust**: Respect VS Code workspace trust settings
- **Permission Checks**: Validate file permissions before access
- **Path Validation**: Prevent path traversal attacks
- **Content Validation**: Validate file content before parsing

#### Data Privacy

- **Local Processing**: All processing happens locally
- **No Network**: No external network calls
- **Telemetry**: Optional local-only telemetry
- **User Control**: User controls all data sharing

### Testing Requirements

#### Unit Tests

- **Coverage**: > 90% code coverage
- **Pure Functions**: Test core logic in isolation
- **Mock Dependencies**: Mock VS Code APIs
- **Edge Cases**: Test error conditions and edge cases

#### Integration Tests

- **End-to-End**: Test complete workflows
- **File System**: Test with real file operations
- **Configuration**: Test with real VS Code settings
- **User Interactions**: Test command execution

#### Performance Tests

- **Large Files**: Test with large .env files
- **Many Files**: Test with many .env files
- **Memory Usage**: Test memory usage under load
- **Response Time**: Test response time under load

### Accessibility

#### User Interface

- **Screen Readers**: Support for screen readers
- **Keyboard Navigation**: Full keyboard navigation support
- **High Contrast**: Support for high contrast themes
- **Color Blindness**: No color-only information

#### Notifications

- **Clear Messages**: Clear, actionable error messages
- **Progress Indicators**: Progress indicators for long operations
- **Status Updates**: Real-time status updates
- **Help Text**: Contextual help and documentation

### Internationalization

#### Localization

- **Multiple Languages**: Support for multiple languages
- **RTL Support**: Support for right-to-left languages
- **Cultural Adaptation**: Adapt to cultural conventions
- **Fallback**: Graceful fallback to English

#### Text Handling

- **Unicode**: Full Unicode support
- **Encoding**: Handle different file encodings
- **Line Endings**: Handle different line ending conventions
- **Special Characters**: Handle special characters in keys and values

This specification provides a comprehensive guide for implementing and maintaining the EnvSync-LE extension while ensuring high quality, performance, and user experience.
