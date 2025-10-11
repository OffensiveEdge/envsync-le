# Changelog

All notable changes to EnvSync-LE will be documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.3.0] - 2025-10-11

### 🎉 Initial Public Release

**EnvSync-LE** - Zero Hassle .env file synchronization across your workspace.

### ✨ Features

#### Core Synchronization

- **Automatic detection**: Discover all .env files in your workspace automatically
- **Real-time sync checking**: Detect missing and extra keys instantly
- **Visual diff support**: Compare .env files side-by-side with syntax highlighting
- **Multiple comparison modes**: Auto, Manual, and Template-based comparison

#### Comparison Modes

- **Auto Mode**: Automatically compare all .env files in workspace
- **Manual Mode**: Select specific files to compare
- **Template Mode**: Use a master .env file as the source of truth

#### Detection & Validation

- **Pattern-based discovery**: Configurable glob patterns for file detection
- **Exclude patterns**: Filter out files you don't want to track
- **Parse error reporting**: Clear feedback on malformed .env files
- **Missing key detection**: Identify keys present in some files but missing in others
- **Extra key detection**: Find keys that don't belong based on comparison

#### User Interface

- **Status bar integration**: At-a-glance sync status with issue counts
- **Notification system**: Configurable alerts (all, errors only, silent)
- **Command palette**: Full keyboard-driven workflow
- **Context menu**: Quick access from Explorer
- **Webview diff viewer**: Beautiful side-by-side comparisons

#### Commands

- **Check Sync**: Manual sync check on demand
- **Compare Files**: Open visual diff for selected .env files
- **Set Template**: Choose a file as comparison template
- **Open Settings**: Quick access to configuration
- **View Help**: In-app documentation

#### Performance & Reliability

- **Debounced file watching**: Smart change detection without excessive checks
- **Concurrent check prevention**: Avoid race conditions
- **Error recovery**: Graceful handling of parse and read errors
- **Memory efficient**: Optimized for large monorepos

#### Enterprise Ready

- **13 languages supported**: Full internationalization (EN, ES, FR, DE, JA, ZH-CN, KO, RU, UK, IT, ID, VI)
- **Virtual workspace support**: Compatible with GitHub Codespaces, Gitpod
- **Untrusted workspace handling**: Safe operation in restricted environments
- **Local-only telemetry**: Privacy-focused with configurable logging

#### Configuration

- **Enable/disable**: Toggle extension on/off
- **Watch patterns**: Customize which files to track
- **Exclude patterns**: Filter files and directories
- **Comparison mode**: Choose Auto, Manual, or Template
- **Template file**: Set master .env file
- **Notification level**: Control alert verbosity
- **Debounce timing**: Adjust file watch sensitivity
- **Status bar**: Show/hide status indicator

### 🔒 Security & Quality

- **Resource management**: Proper cleanup of watchers, timers, and disposables
- **Error handling**: Comprehensive error handling with user feedback
- **Disposal guards**: Prevention of use-after-disposal issues
- **Code quality**: Zero linter warnings, 121 passing tests, strict TypeScript

### 🚀 Part of the LE Family

EnvSync-LE is part of a growing family of developer productivity tools:

- [Strings-LE](https://open-vsx.org/extension/nolindnaidoo/string-le) - String extraction from structured files
- [Numbers-LE](https://open-vsx.org/extension/nolindnaidoo/numbers-le) - Numeric data extraction
- [Colors-LE](https://open-vsx.org/extension/nolindnaidoo/colors-le) - Color analysis
- [Dates-LE](https://open-vsx.org/extension/nolindnaidoo/dates-le) - Date extraction
- [Paths-LE](https://open-vsx.org/extension/nolindnaidoo/paths-le) - File path analysis
- [URLs-LE](https://open-vsx.org/extension/nolindnaidoo/urls-le) - URL extraction

Each tool follows the same philosophy: **Zero Hassle, Maximum Productivity**.
