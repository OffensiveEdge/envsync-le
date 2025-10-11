# EnvSync-LE Privacy Policy

This document outlines the privacy practices and data handling policies for the EnvSync-LE VS Code extension.

## Privacy Philosophy

EnvSync-LE is designed with privacy as a core principle. We believe that your development environment and configuration data should remain private and under your control at all times.

## Data Collection

### What We Collect

EnvSync-LE collects **no personal data** by default. The extension operates entirely locally within your VS Code environment.

### Optional Telemetry

If enabled, EnvSync-LE collects minimal, anonymous usage data:

#### Telemetry Data (Optional)

- **Command Usage**: Which commands are executed (e.g., "showIssues", "compareSelected")
- **Sync Events**: Sync check results and status (e.g., "in-sync", "missing-keys")
- **Performance Metrics**: Operation timing and file counts (anonymized)
- **Error Events**: Error types and frequencies (no sensitive data)

#### What We Don't Collect

- **File Contents**: Never reads or transmits .env file contents
- **Environment Variables**: Never accesses actual environment variable values
- **Personal Information**: No names, emails, or identifying information
- **Network Data**: No network requests or external communications
- **Workspace Information**: No project names, paths, or workspace details

### Telemetry Control

Telemetry is **disabled by default** and can be controlled via settings:

```json
{
  "envsync-le.telemetryEnabled": false
}
```

## Data Processing

### Local Processing Only

All data processing happens locally on your machine:

- **File Parsing**: .env files are parsed locally
- **Sync Detection**: Comparisons are performed locally
- **Configuration**: Settings are processed locally
- **UI Updates**: All user interface updates are local

### No External Communications

EnvSync-LE makes **no network requests**:

- **No Internet Connection**: Works entirely offline
- **No External APIs**: No calls to external services
- **No Data Transmission**: No data is sent over the network
- **No Updates**: No automatic updates or version checks

### Data Storage

All data is stored locally:

- **Configuration**: Stored in VS Code settings
- **Cache**: Temporary cache stored in VS Code extension directory
- **Logs**: Local logs in VS Code Output channel
- **No Persistent Storage**: No permanent data storage

## Data Security

### File Access

EnvSync-LE only accesses files you explicitly allow:

- **Workspace Trust**: Respects VS Code workspace trust settings
- **File Permissions**: Only accesses files with proper permissions
- **Pattern Matching**: Only processes files matching configured patterns
- **Exclusion Support**: Supports excluding sensitive files

### Data Protection

Your data is protected through:

- **Local Processing**: All processing happens on your machine
- **No Transmission**: No data leaves your machine
- **Minimal Access**: Only accesses necessary files
- **Secure Parsing**: Safe parsing with error handling

### Workspace Trust

EnvSync-LE respects VS Code's workspace trust:

- **Trusted Workspaces**: Only works in trusted workspaces
- **Untrusted Workspaces**: Disabled in untrusted workspaces
- **User Control**: User controls workspace trust settings
- **Security**: Prevents execution in untrusted environments

## Configuration Privacy

### Settings Privacy

Configuration settings are private:

- **Local Storage**: Settings stored locally in VS Code
- **No Transmission**: Settings never transmitted externally
- **User Control**: User has full control over all settings
- **No Default Sharing**: No settings are shared by default

### Sensitive Settings

Sensitive configuration options:

- **File Patterns**: Watch and exclude patterns are local
- **Template Files**: Template file paths are local
- **Ignore Lists**: Temporary ignore lists are local
- **Comparison Modes**: Comparison settings are local

## Telemetry Details

### When Telemetry is Enabled

If you enable telemetry (`envsync-le.telemetryEnabled: true`), the following data is collected:

#### Event Types

```typescript
// Command usage (anonymized)
{
  event: 'command',
  data: { name: 'showIssues' }
}

// Sync check results (anonymized)
{
  event: 'sync-check',
  data: {
    status: 'in-sync',
    fileCount: 3,
    missingKeysCount: 0
  }
}

// Performance metrics (anonymized)
{
  event: 'performance',
  data: {
    syncCheckTime: 45,
    fileCount: 5,
    memoryUsage: 1024
  }
}
```

#### Data Anonymization

All telemetry data is anonymized:

- **No File Paths**: File paths are not included
- **No File Contents**: File contents are never included
- **No Personal Data**: No personal or identifying information
- **Aggregated Metrics**: Only aggregated, non-identifying metrics

### Telemetry Storage

Telemetry data is stored locally:

- **Output Channel**: Logged to VS Code Output channel
- **No External Storage**: No data sent to external servers
- **Local Only**: All data remains on your machine
- **User Accessible**: You can view all telemetry data

### Disabling Telemetry

You can disable telemetry at any time:

```json
{
  "envsync-le.telemetryEnabled": false
}
```

## Third-Party Dependencies

### Dependency Privacy

EnvSync-LE uses minimal dependencies:

- **VS Code API**: Uses only VS Code's public API
- **Node.js**: Uses standard Node.js modules
- **No External Libraries**: No external libraries that collect data
- **Open Source**: All dependencies are open source

### Dependency Audit

Regular dependency audits ensure privacy:

- **Security Audits**: Regular security audits of dependencies
- **Privacy Reviews**: Privacy reviews of all dependencies
- **Minimal Dependencies**: Keep dependencies to a minimum
- **Transparent Dependencies**: All dependencies are transparent

## User Rights

### Data Control

You have full control over your data:

- **Enable/Disable**: Control telemetry and data collection
- **View Data**: View all collected data in Output channel
- **Delete Data**: Clear all local data by disabling extension
- **Export Data**: Export configuration settings if needed

### Transparency

We are transparent about data handling:

- **Open Source**: Extension is open source
- **Documentation**: Comprehensive privacy documentation
- **No Hidden Features**: No hidden data collection
- **User Consent**: Clear consent for any data collection

### Data Portability

Your data is portable:

- **Local Storage**: All data stored locally
- **Standard Formats**: Uses standard VS Code settings format
- **Exportable**: Configuration can be exported
- **No Lock-in**: No vendor lock-in for your data

## Compliance

### Privacy Regulations

EnvSync-LE is designed to comply with privacy regulations:

- **GDPR**: Compliant with General Data Protection Regulation
- **CCPA**: Compliant with California Consumer Privacy Act
- **Local Laws**: Compliant with local privacy laws
- **Industry Standards**: Follows industry privacy best practices

### Data Minimization

We follow data minimization principles:

- **Minimal Collection**: Collect only necessary data
- **Local Processing**: Process data locally
- **No Retention**: No long-term data retention
- **User Control**: User controls all data collection

## Security Measures

### Code Security

The extension is secure by design:

- **Open Source**: Code is open source and auditable
- **Security Reviews**: Regular security reviews
- **Dependency Updates**: Regular dependency updates
- **Vulnerability Scanning**: Regular vulnerability scanning

### Runtime Security

Runtime security measures:

- **Sandboxed**: Runs in VS Code's sandboxed environment
- **Limited Permissions**: Limited file system permissions
- **No Network Access**: No network access capabilities
- **Error Handling**: Secure error handling

## Contact and Support

### Privacy Questions

If you have privacy questions:

- **GitHub Issues**: Open an issue on GitHub
- **GitHub Discussions**: Use GitHub Discussions
- **Documentation**: Check comprehensive documentation
- **Source Code**: Review the open source code

### Privacy Concerns

If you have privacy concerns:

- **Report Issues**: Report privacy issues on GitHub
- **Request Changes**: Request privacy-related changes
- **Provide Feedback**: Provide feedback on privacy practices
- **Contribute**: Contribute to privacy improvements

## Updates and Changes

### Privacy Policy Updates

This privacy policy may be updated:

- **Version Control**: Changes are version controlled
- **Notification**: Users notified of significant changes
- **Transparency**: All changes are transparent
- **User Consent**: User consent for significant changes

### Extension Updates

Extension updates maintain privacy:

- **No New Data Collection**: Updates don't add new data collection
- **Privacy Preservation**: Privacy practices are preserved
- **User Control**: User control is maintained
- **Transparency**: All changes are transparent

## Summary

EnvSync-LE is designed with privacy as a core principle:

- **No Data Collection**: No personal data is collected by default
- **Local Processing**: All processing happens locally
- **No Network Access**: No external communications
- **User Control**: Full user control over all data
- **Transparency**: Complete transparency about data handling
- **Security**: Secure by design with minimal attack surface

Your privacy is our priority, and we are committed to maintaining the highest standards of data protection and user privacy.
