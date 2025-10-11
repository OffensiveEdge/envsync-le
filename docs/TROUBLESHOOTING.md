# EnvSync-LE Troubleshooting

Structured guide to diagnose and resolve common issues. For full behavior see `SPECIFICATION.md` and `CONFIGURATION.md`.

## Quick Mapping (Symptom → Fix)

- Extension not working → check enabled setting, workspace trust, reload window
- No .env files detected → check watch patterns, file locations, exclusion patterns
- Sync status incorrect → check comparison mode, template file, case sensitivity
- Too many notifications → adjust notification level, exclude patterns
- Performance issues → increase debounce, narrow watch patterns, add exclusions
- Commands missing → check file selection, workspace trust, reload window
- Parse errors → check file format, enable parse error display
- Status bar not showing → check status bar enabled setting

---

## 1) Extension Activation & Basic Functionality

### Extension Not Working

**Symptoms**: No status bar item, commands not available, no file detection

**Diagnosis**:

1. Check `envsync-le.enabled` setting
2. Verify workspace trust is enabled
3. Check if .env files exist in workspace
4. Verify file patterns match your files

**Solutions**:

```json
{
  "envsync-le.enabled": true
}
```

- Trust workspace: "Workspace: Manage Workspace Trust"
- Reload window: "Developer: Reload Window"
- Check file patterns: `envsync-le.watchPatterns`

**Related Settings**:

- `envsync-le.enabled`
- `envsync-le.watchPatterns`
- `envsync-le.excludePatterns`

### Commands Not Available

**Symptoms**: Commands don't appear in Command Palette or context menu

**Diagnosis**:

1. Check extension is enabled
2. Verify workspace trust
3. Check if .env files are selected (for context menu)
4. Verify file extensions match patterns

**Solutions**:

- Enable extension: `envsync-le.enabled: true`
- Trust workspace
- Select .env files for context menu commands
- Check file patterns include your files

**Related Settings**:

- `envsync-le.enabled`
- `envsync-le.watchPatterns`

### Status Bar Not Showing

**Symptoms**: No EnvSync-LE status bar item

**Diagnosis**:

1. Check status bar enabled setting
2. Verify extension is enabled
3. Check if .env files exist

**Solutions**:

```json
{
  "envsync-le.statusBar.enabled": true
}
```

- Enable extension
- Ensure .env files exist in workspace

**Related Settings**:

- `envsync-le.statusBar.enabled`
- `envsync-le.enabled`

---

## 2) File Detection & Patterns

### No .env Files Detected

**Symptoms**: "No .env files found" message, empty sync report

**Diagnosis**:

1. Check watch patterns match your files
2. Verify exclusion patterns aren't too broad
3. Check file locations and names
4. Verify file extensions

**Solutions**:

```json
{
  "envsync-le.watchPatterns": ["**/.env*", "**/.env.*", "config/**/.env*"],
  "envsync-le.excludePatterns": [".env.*.local"]
}
```

- Adjust patterns to match your file structure
- Remove overly broad exclusion patterns
- Check file names and locations

**Related Settings**:

- `envsync-le.watchPatterns`
- `envsync-le.excludePatterns`

### Files Not Being Watched

**Symptoms**: Changes to .env files don't trigger sync checks

**Diagnosis**:

1. Check file patterns
2. Verify exclusion patterns
3. Check debounce setting
4. Verify file system permissions

**Solutions**:

- Adjust watch patterns to include your files
- Remove exclusion patterns that block your files
- Reduce debounce for faster response
- Check file permissions

**Related Settings**:

- `envsync-le.watchPatterns`
- `envsync-le.excludePatterns`
- `envsync-le.debounceMs`

### Too Many Files Being Watched

**Symptoms**: Performance issues, excessive notifications

**Diagnosis**:

1. Check watch patterns are too broad
2. Verify exclusion patterns are missing
3. Check debounce setting is too low

**Solutions**:

```json
{
  "envsync-le.watchPatterns": ["**/.env*"],
  "envsync-le.excludePatterns": [
    ".env.*.local",
    "node_modules/**/.env*",
    "dist/**/.env*",
    "build/**/.env*"
  ],
  "envsync-le.debounceMs": 2000
}
```

- Narrow watch patterns
- Add exclusion patterns for large directories
- Increase debounce delay

**Related Settings**:

- `envsync-le.watchPatterns`
- `envsync-le.excludePatterns`
- `envsync-le.debounceMs`

---

## 3) Sync Detection & Comparison

### Sync Status Incorrect

**Symptoms**: Wrong sync status, missing keys not detected

**Diagnosis**:

1. Check comparison mode setting
2. Verify template file (if using template mode)
3. Check case sensitivity setting
4. Verify file parsing

**Solutions**:

```json
{
  "envsync-le.comparisonMode": "auto",
  "envsync-le.caseSensitive": true,
  "envsync-le.templateFile": ".env.example"
}
```

- Switch comparison mode
- Set correct template file
- Adjust case sensitivity
- Check file format

**Related Settings**:

- `envsync-le.comparisonMode`
- `envsync-le.templateFile`
- `envsync-le.caseSensitive`

### Template Mode Not Working

**Symptoms**: Template comparison not working, wrong reference file

**Diagnosis**:

1. Check template file setting
2. Verify template file exists
3. Check comparison mode
4. Verify template file is included in watch patterns

**Solutions**:

```json
{
  "envsync-le.comparisonMode": "template",
  "envsync-le.templateFile": ".env.example"
}
```

- Set correct template file path
- Ensure template file exists
- Switch to template mode
- Include template file in watch patterns

**Related Settings**:

- `envsync-le.comparisonMode`
- `envsync-le.templateFile`
- `envsync-le.watchPatterns`

### Case Sensitivity Issues

**Symptoms**: Keys not matching due to case differences

**Diagnosis**:

1. Check case sensitivity setting
2. Verify key casing in files
3. Check comparison mode

**Solutions**:

```json
{
  "envsync-le.caseSensitive": false
}
```

- Disable case sensitivity for case-insensitive comparison
- Ensure consistent casing in files
- Check comparison mode

**Related Settings**:

- `envsync-le.caseSensitive`
- `envsync-le.comparisonMode`

---

## 4) Notifications & User Interface

### Too Many Notifications

**Symptoms**: Excessive notifications, notification spam

**Diagnosis**:

1. Check notification level setting
2. Verify exclusion patterns
3. Check debounce setting
4. Verify file patterns

**Solutions**:

```json
{
  "envsync-le.notificationLevel": "important"
}
```

- Reduce notification level
- Add exclusion patterns
- Increase debounce delay
- Narrow file patterns

**Related Settings**:

- `envsync-le.notificationLevel`
- `envsync-le.excludePatterns`
- `envsync-le.debounceMs`

### No Notifications

**Symptoms**: No notifications, silent operation

**Diagnosis**:

1. Check notification level setting
2. Verify extension is enabled
3. Check if issues exist

**Solutions**:

```json
{
  "envsync-le.notificationLevel": "all"
}
```

- Increase notification level
- Enable extension
- Check if sync issues exist

**Related Settings**:

- `envsync-le.notificationLevel`
- `envsync-le.enabled`

### Status Bar Not Updating

**Symptoms**: Status bar shows old information, not reflecting current state

**Diagnosis**:

1. Check status bar enabled setting
2. Verify extension is working
3. Check if sync check is running

**Solutions**:

- Enable status bar
- Reload window
- Run "Show Issues" command
- Check extension logs

**Related Settings**:

- `envsync-le.statusBar.enabled`
- `envsync-le.enabled`

---

## 5) Performance Issues

### Slow Sync Checks

**Symptoms**: Long delays, UI freezing, high CPU usage

**Diagnosis**:

1. Check debounce setting
2. Verify file patterns
3. Check exclusion patterns
4. Verify file sizes

**Solutions**:

```json
{
  "envsync-le.debounceMs": 2000,
  "envsync-le.excludePatterns": ["node_modules/**/.env*", "dist/**/.env*", "build/**/.env*"]
}
```

- Increase debounce delay
- Add exclusion patterns for large directories
- Narrow file patterns
- Check file sizes

**Related Settings**:

- `envsync-le.debounceMs`
- `envsync-le.excludePatterns`
- `envsync-le.watchPatterns`

### High Memory Usage

**Symptoms**: High memory consumption, slow performance

**Diagnosis**:

1. Check file patterns
2. Verify exclusion patterns
3. Check file sizes
4. Verify number of files

**Solutions**:

- Add exclusion patterns for large directories
- Narrow file patterns
- Check file sizes
- Limit number of files being watched

**Related Settings**:

- `envsync-le.excludePatterns`
- `envsync-le.watchPatterns`
- `envsync-le.compareOnlyFiles`

### File System Errors

**Symptoms**: File system errors, permission issues

**Diagnosis**:

1. Check file permissions
2. Verify file system access
3. Check network drives
4. Verify file locks

**Solutions**:

- Check file permissions
- Verify file system access
- Avoid network drives if possible
- Check for file locks

**Related Settings**:

- File system permissions
- Network drive settings

---

## 6) Parsing & File Format Issues

### Parse Errors

**Symptoms**: Parse errors, malformed file warnings

**Diagnosis**:

1. Check file format
2. Verify file encoding
3. Check for malformed lines
4. Verify file syntax

**Solutions**:

- Check file format and syntax
- Verify file encoding (UTF-8 recommended)
- Fix malformed lines
- Validate file syntax

**Related Settings**:

- `envsync-le.ignoreComments`
- `envsync-le.caseSensitive`

### File Encoding Issues

**Symptoms**: Characters not displaying correctly, parse errors

**Diagnosis**:

1. Check file encoding
2. Verify character set
3. Check for BOM
4. Verify line endings

**Solutions**:

- Use UTF-8 encoding
- Remove BOM if present
- Use consistent line endings
- Check character set

**Related Settings**:

- File encoding settings
- Line ending settings

### Large File Issues

**Symptoms**: Slow parsing, memory issues, timeouts

**Diagnosis**:

1. Check file size
2. Verify file complexity
3. Check memory usage
4. Verify parsing performance

**Solutions**:

- Split large files
- Optimize file structure
- Check memory usage
- Consider file size limits

**Related Settings**:

- File size limits
- Memory usage settings

---

## 7) Configuration Issues

### Settings Not Applied

**Symptoms**: Settings changes not taking effect

**Diagnosis**:

1. Check workspace vs user settings
2. Verify setting syntax
3. Check for configuration errors
4. Verify setting names

**Solutions**:

- Check workspace vs user settings precedence
- Verify JSON syntax
- Check for configuration errors
- Verify setting names and values

**Related Settings**:

- All EnvSync-LE settings
- VS Code settings precedence

### Configuration Errors

**Symptoms**: Invalid settings, configuration warnings

**Diagnosis**:

1. Check JSON syntax
2. Verify setting values
3. Check for typos
4. Verify setting types

**Solutions**:

- Fix JSON syntax errors
- Verify setting values
- Check for typos in setting names
- Verify setting types match schema

**Related Settings**:

- All EnvSync-LE settings
- VS Code settings schema

### Workspace vs User Settings

**Symptoms**: Settings not working as expected, conflicts

**Diagnosis**:

1. Check settings precedence
2. Verify workspace settings
3. Check user settings
4. Verify setting inheritance

**Solutions**:

- Understand settings precedence (workspace overrides user)
- Check both workspace and user settings
- Verify setting inheritance
- Clear conflicting settings

**Related Settings**:

- All EnvSync-LE settings
- VS Code settings precedence

---

## 8) Advanced Troubleshooting

### Debug Logging

**Enable debug logging**:

```json
{
  "envsync-le.telemetryEnabled": true
}
```

- Check Output channel: "View → Output → EnvSync-LE"
- Look for detailed error information
- Check for configuration issues

### Reset Configuration

**Reset to defaults**:

1. Remove all `envsync-le.*` settings
2. Reload VS Code window
3. Extension will use defaults
4. Reconfigure as needed

### Check Extension Logs

**View extension logs**:

1. Open Output panel (View → Output)
2. Select "EnvSync-LE" from dropdown
3. Look for error messages and warnings
4. Check for configuration issues

### Verify File System

**Check file system**:

1. Verify file permissions
2. Check file system type
3. Verify network drive access
4. Check for file locks

### Performance Profiling

**Profile performance**:

1. Enable telemetry
2. Check Output channel for performance metrics
3. Monitor memory usage
4. Check CPU usage

---

## 9) Common Solutions

### Quick Fixes

1. **Reload Window**: "Developer: Reload Window"
2. **Check Settings**: Verify `envsync-le.enabled: true`
3. **Trust Workspace**: Enable workspace trust
4. **Check Files**: Verify .env files exist and are readable

### Configuration Reset

```json
{
  "envsync-le.enabled": true,
  "envsync-le.watchPatterns": ["**/.env*"],
  "envsync-le.excludePatterns": [".env.*.local"],
  "envsync-le.notificationLevel": "important",
  "envsync-le.statusBar.enabled": true,
  "envsync-le.debounceMs": 1000,
  "envsync-le.comparisonMode": "auto"
}
```

### Performance Optimization

```json
{
  "envsync-le.debounceMs": 2000,
  "envsync-le.excludePatterns": [
    ".env.*.local",
    "node_modules/**/.env*",
    "dist/**/.env*",
    "build/**/.env*"
  ],
  "envsync-le.notificationLevel": "important"
}
```

### Development Environment

```json
{
  "envsync-le.enabled": true,
  "envsync-le.watchPatterns": ["**/.env*"],
  "envsync-le.excludePatterns": [".env.production", ".env.staging"],
  "envsync-le.notificationLevel": "all",
  "envsync-le.comparisonMode": "auto",
  "envsync-le.debounceMs": 500
}
```

---

## 10) Getting Help

### Documentation

- **Architecture**: See `ARCHITECTURE.md` for technical details
- **Specification**: See `SPECIFICATION.md` for behavior details
- **Configuration**: See `CONFIGURATION.md` for settings guide
- **Commands**: See `COMMANDS.md` for command reference

### Support

- **GitHub Issues**: Report bugs and feature requests
- **GitHub Discussions**: Ask questions and get help
- **Documentation**: Check comprehensive documentation
- **Logs**: Check Output channel for detailed information

### Reporting Issues

When reporting issues, include:

1. **VS Code Version**: Version and platform
2. **Extension Version**: EnvSync-LE version
3. **Configuration**: Relevant settings
4. **Logs**: Output channel logs
5. **Steps to Reproduce**: Detailed steps
6. **Expected vs Actual**: Expected and actual behavior

This troubleshooting guide provides comprehensive solutions for common EnvSync-LE issues and helps users diagnose and resolve problems effectively.
