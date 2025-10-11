# EnvSync-LE Configuration Guide

This document provides a comprehensive guide to configuring EnvSync-LE, including all available settings, their purposes, and recommended values.

## Overview

EnvSync-LE uses VS Code's configuration system to manage settings. All settings are prefixed with `envsync-le.` and can be configured at the workspace or user level.

## Quick Configuration

### Basic Setup

```json
{
  "envsync-le.enabled": true,
  "envsync-le.watchPatterns": ["**/.env*"],
  "envsync-le.notificationLevel": "important"
}
```

### Advanced Setup

```json
{
  "envsync-le.enabled": true,
  "envsync-le.watchPatterns": ["**/.env*", "**/.env.*"],
  "envsync-le.excludePatterns": [".env.*.local", ".env.temp"],
  "envsync-le.notificationLevel": "all",
  "envsync-le.comparisonMode": "template",
  "envsync-le.templateFile": ".env.example",
  "envsync-le.debounceMs": 500,
  "envsync-le.caseSensitive": false
}
```

## Configuration Categories

### Core Settings

#### `envsync-le.enabled`

- **Type**: `boolean`
- **Default**: `true`
- **Description**: Enable or disable the EnvSync-LE extension
- **Usage**: Set to `false` to completely disable the extension
- **Example**: `"envsync-le.enabled": false`

#### `envsync-le.watchPatterns`

- **Type**: `array` of `string`
- **Default**: `[".env*"]`
- **Description**: File patterns to watch for .env files
- **Usage**: Use glob patterns to specify which files to monitor
- **Examples**:
  ```json
  "envsync-le.watchPatterns": [
    "**/.env*",
    "**/.env.*",
    "config/**/.env*"
  ]
  ```

#### `envsync-le.excludePatterns`

- **Type**: `array` of `string`
- **Default**: `[".env.*.local"]`
- **Description**: File patterns to exclude from watching
- **Usage**: Use glob patterns to exclude specific files or directories
- **Examples**:
  ```json
  "envsync-le.excludePatterns": [
    ".env.*.local",
    ".env.temp",
    "node_modules/**/.env*",
    "dist/**/.env*"
  ]
  ```

### Notification Settings

#### `envsync-le.notificationLevel`

- **Type**: `string`
- **Default**: `"important"`
- **Enum**: `["all", "important", "silent"]`
- **Description**: Control the verbosity of notifications
- **Options**:
  - `"all"`: Show all notifications (info, warnings, errors)
  - `"important"`: Show only warnings and errors
  - `"silent"`: Suppress all notifications
- **Example**: `"envsync-le.notificationLevel": "all"`

### User Interface Settings

#### `envsync-le.statusBar.enabled`

- **Type**: `boolean`
- **Default**: `true`
- **Description**: Show the status bar item
- **Usage**: Set to `false` to hide the status bar item
- **Example**: `"envsync-le.statusBar.enabled": false`

### Performance Settings

#### `envsync-le.debounceMs`

- **Type**: `number`
- **Default**: `1000`
- **Minimum**: `100`
- **Description**: Debounce delay for file change events (in milliseconds)
- **Usage**: Increase for better performance with many files, decrease for faster response
- **Examples**:
  - `"envsync-le.debounceMs": 500` (faster response)
  - `"envsync-le.debounceMs": 2000` (better performance)

### Parsing Settings

#### `envsync-le.ignoreComments`

- **Type**: `boolean`
- **Default**: `true`
- **Description**: Ignore comment lines in .env files
- **Usage**: Set to `false` to include comment lines in parsing
- **Example**: `"envsync-le.ignoreComments": false`

#### `envsync-le.caseSensitive`

- **Type**: `boolean`
- **Default**: `true`
- **Description**: Case-sensitive key comparison
- **Usage**: Set to `false` for case-insensitive key comparison
- **Example**: `"envsync-le.caseSensitive": false`

### Comparison Settings

#### `envsync-le.comparisonMode`

- **Type**: `string`
- **Default**: `"auto"`
- **Enum**: `["auto", "manual", "template"]`
- **Description**: Comparison mode for sync detection
- **Options**:
  - `"auto"`: Compare all files against each other
  - `"manual"`: Compare only selected files
  - `"template"`: Compare all files against a template file
- **Example**: `"envsync-le.comparisonMode": "template"`

#### `envsync-le.templateFile`

- **Type**: `string`
- **Default**: `undefined`
- **Description**: Template file for template comparison mode
- **Usage**: Specify the relative path to the template file
- **Example**: `"envsync-le.templateFile": ".env.example"`

#### `envsync-le.compareOnlyFiles`

- **Type**: `array` of `string`
- **Default**: `[]`
- **Description**: Only compare these specific files
- **Usage**: Specify relative paths to files to compare
- **Example**: `"envsync-le.compareOnlyFiles": [".env", ".env.production"]`

#### `envsync-le.temporaryIgnore`

- **Type**: `array` of `string`
- **Default**: `[]`
- **Description**: Temporarily ignore these files from sync checking
- **Usage**: Specify relative paths to files to ignore
- **Example**: `"envsync-le.temporaryIgnore": [".env.local"]`

### Safety Settings

#### `envsync-le.safety.enabled`

- **Type**: `boolean`
- **Default**: `false`
- **Description**: Enable safety checks for large files and operations
- **Usage**: Set to `true` to enable safety warnings and thresholds
- **Example**: `"envsync-le.safety.enabled": true`

#### `envsync-le.safety.fileSizeWarnBytes`

- **Type**: `number`
- **Default**: `1048576` (1MB)
- **Minimum**: `1024`
- **Description**: Warn when file size exceeds this threshold (in bytes)
- **Usage**: Adjust based on your typical .env file sizes
- **Example**: `"envsync-le.safety.fileSizeWarnBytes": 2097152` (2MB)

#### `envsync-le.safety.maxFilesWarn`

- **Type**: `number`
- **Default**: `50`
- **Minimum**: `1`
- **Description**: Warn when number of files exceeds this threshold
- **Usage**: Adjust for large workspaces with many .env files
- **Example**: `"envsync-le.safety.maxFilesWarn": 100`

#### `envsync-le.safety.maxTotalSizeWarn`

- **Type**: `number`
- **Default**: `5242880` (5MB)
- **Minimum**: `1048576`
- **Description**: Warn when total size of all files exceeds this threshold (in bytes)
- **Usage**: Adjust for workspaces with many large .env files
- **Example**: `"envsync-le.safety.maxTotalSizeWarn": 10485760` (10MB)

#### `envsync-le.safety.maxProcessingTimeWarn`

- **Type**: `number`
- **Default**: `5000` (5 seconds)
- **Minimum**: `1000`
- **Description**: Warn when processing time exceeds this threshold (in milliseconds)
- **Usage**: Adjust for performance-sensitive environments
- **Example**: `"envsync-le.safety.maxProcessingTimeWarn": 10000` (10 seconds)

### Telemetry Settings

#### `envsync-le.telemetryEnabled`

- **Type**: `boolean`
- **Default**: `false`
- **Description**: Enable local-only telemetry
- **Usage**: Set to `true` to enable local telemetry logging
- **Example**: `"envsync-le.telemetryEnabled": true`

## Configuration Scenarios

### Development Environment

```json
{
  "envsync-le.enabled": true,
  "envsync-le.watchPatterns": [".env*"],
  "envsync-le.excludePatterns": [".env.production", ".env.staging"],
  "envsync-le.notificationLevel": "all",
  "envsync-le.comparisonMode": "auto",
  "envsync-le.debounceMs": 500,
  "envsync-le.caseSensitive": true,
  "envsync-le.safety.enabled": false
}
```

### Production Environment

```json
{
  "envsync-le.enabled": true,
  "envsync-le.watchPatterns": [".env*"],
  "envsync-le.excludePatterns": [".env.*.local", ".env.temp"],
  "envsync-le.notificationLevel": "important",
  "envsync-le.comparisonMode": "template",
  "envsync-le.templateFile": ".env.example",
  "envsync-le.debounceMs": 1000,
  "envsync-le.caseSensitive": true,
  "envsync-le.safety.enabled": true,
  "envsync-le.safety.fileSizeWarnBytes": 1048576,
  "envsync-le.safety.maxFilesWarn": 50
}
```

### Large Workspace

```json
{
  "envsync-le.enabled": true,
  "envsync-le.watchPatterns": [".env*"],
  "envsync-le.excludePatterns": [
    ".env.*.local",
    "node_modules/**/.env*",
    "dist/**/.env*",
    "build/**/.env*"
  ],
  "envsync-le.notificationLevel": "important",
  "envsync-le.debounceMs": 2000,
  "envsync-le.compareOnlyFiles": [".env", ".env.production", ".env.staging"],
  "envsync-le.safety.enabled": true,
  "envsync-le.safety.maxFilesWarn": 100,
  "envsync-le.safety.maxTotalSizeWarn": 10485760
}
```

### Template-Based Workflow

```json
{
  "envsync-le.enabled": true,
  "envsync-le.watchPatterns": [".env*"],
  "envsync-le.excludePatterns": [".env.*.local"],
  "envsync-le.notificationLevel": "all",
  "envsync-le.comparisonMode": "template",
  "envsync-le.templateFile": ".env.example",
  "envsync-le.caseSensitive": false,
  "envsync-le.safety.enabled": false
}
```

## Configuration Validation

### Automatic Validation

- **Type Checking**: VS Code validates types automatically
- **Enum Values**: Invalid enum values are rejected
- **Minimum Values**: Values below minimum are adjusted
- **Default Fallback**: Invalid values fall back to defaults

### Manual Validation

- **Settings UI**: Use VS Code Settings UI for validation
- **JSON Schema**: JSON schema provides IntelliSense
- **Error Messages**: Clear error messages for invalid settings

## Configuration Migration

### Version Updates

- **Backward Compatibility**: Old settings are supported
- **Migration Notes**: Check changelog for migration instructions
- **Default Updates**: New defaults are applied automatically
- **Deprecation**: Deprecated settings are supported for one major version

### Workspace vs User Settings

- **Workspace Settings**: Apply to specific workspace
- **User Settings**: Apply to all workspaces
- **Precedence**: Workspace settings override user settings
- **Inheritance**: User settings provide defaults for workspace settings

## Troubleshooting Configuration

### Common Issues

#### Extension Not Working

- **Check**: `envsync-le.enabled` is set to `true`
- **Check**: Workspace trust is enabled
- **Check**: File patterns match your .env files
- **Solution**: Reload VS Code window

#### Too Many Notifications

- **Check**: `envsync-le.notificationLevel` setting
- **Solution**: Set to `"important"` or `"silent"`
- **Check**: Exclude patterns are correct
- **Solution**: Add more exclusion patterns

#### Performance Issues

- **Check**: `envsync-le.debounceMs` setting
- **Solution**: Increase debounce delay
- **Check**: Watch patterns are too broad
- **Solution**: Narrow down watch patterns
- **Check**: Exclude patterns are missing
- **Solution**: Add exclusion patterns for large directories

#### Sync Detection Not Working

- **Check**: `envsync-le.comparisonMode` setting
- **Check**: `envsync-le.templateFile` is correct (if using template mode)
- **Check**: File patterns include your .env files
- **Solution**: Run "EnvSync-LE: Show Issues" command

### Configuration Debugging

#### Enable Debug Logging

```json
{
  "envsync-le.telemetryEnabled": true
}
```

#### Check Configuration

1. Open VS Code Settings (Ctrl/Cmd + ,)
2. Search for "envsync-le"
3. Verify all settings are correct
4. Check workspace vs user settings

#### Reset Configuration

1. Remove all `envsync-le.*` settings
2. Reload VS Code window
3. Extension will use defaults
4. Reconfigure as needed

## Best Practices

### Performance

- **Use Exclusion Patterns**: Exclude large directories (node_modules, dist, build)
- **Adjust Debounce**: Increase debounce for large workspaces
- **Limit Watch Patterns**: Use specific patterns instead of broad ones
- **Use Compare Only Files**: Limit comparison to specific files

### Security

- **Exclude Sensitive Files**: Exclude files with sensitive data
- **Use Template Mode**: Use template mode for consistent environments
- **Case Sensitivity**: Use case-sensitive comparison for security
- **Workspace Trust**: Only enable in trusted workspaces

### Maintenance

- **Regular Updates**: Keep extension updated
- **Configuration Review**: Review configuration periodically
- **Documentation**: Document custom configurations
- **Testing**: Test configuration changes in development

### Team Collaboration

- **Workspace Settings**: Use workspace settings for team consistency
- **Template Files**: Use template files for environment consistency
- **Documentation**: Document team-specific configurations
- **Version Control**: Include configuration in version control

This configuration guide provides comprehensive information for setting up and maintaining EnvSync-LE in various environments and use cases.
