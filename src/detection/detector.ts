import { readConfig } from '../config/config'
import type { Configuration, FileSystem } from '../interfaces'
import type { Notifier } from '../interfaces/notifier'
import type { StatusBar } from '../interfaces/statusBar'
import type { Telemetry } from '../interfaces/telemetry'
import type { DotenvFile, ParseError, ParseResult, SyncReport } from '../types'
import { compareFiles } from './comparator'
import { detectFileType, parseDotenvFile, shouldExcludeFile } from './parser'

export interface Detector {
  checkSync(): Promise<SyncReport>
  checkSyncForFiles(filePaths: readonly string[]): Promise<SyncReport>
  dispose(): void
}

export function createDetector(
  deps: Readonly<{
    telemetry: Telemetry
    notifier: Notifier
    statusBar: StatusBar
    configuration: Configuration
    fileSystem: FileSystem
  }>,
): Detector {
  const { telemetry, notifier, statusBar, configuration, fileSystem } = deps

  async function checkSync(): Promise<SyncReport> {
    const config = readConfig(configuration)

    if (!config.enabled) {
      return createDisabledReport()
    }

    try {
      return await performSyncCheck(config)
    } catch (error) {
      return handleSyncCheckError(error, config)
    }
  }

  async function performSyncCheck(config: ReturnType<typeof readConfig>): Promise<SyncReport> {
    const { files, errors } = await discoverDotenvFiles(
      fileSystem,
      config.watchPatterns,
      config.excludePatterns,
      config,
    )

    const compareOptions = buildCompareOptions(files, config)
    const report = compareFiles(files, compareOptions)
    const finalReport = { ...report, errors: Object.freeze(errors) }

    updateUI(finalReport)
    notifyUser(finalReport, errors, config)
    trackTelemetry(finalReport, files)

    return finalReport
  }

  function buildCompareOptions(
    files: DotenvFile[],
    config: ReturnType<typeof readConfig>,
  ): { mode: 'auto' | 'template'; templatePath?: string } {
    const isTemplateMode = config.comparisonMode === 'template'

    if (!isTemplateMode || !config.templateFile) {
      return { mode: 'auto' }
    }

    const templatePath = findTemplatePath(files, config.templateFile)

    if (templatePath) {
      return {
        mode: 'template',
        templatePath,
      }
    }

    return {
      mode: 'template',
    }
  }

  function findTemplatePath(files: DotenvFile[], templateFile: string): string | undefined {
    const template = files.find((file) => fileSystem.asRelativePath(file.path) === templateFile)
    return template?.path
  }

  function updateUI(report: SyncReport): void {
    const issueCount = report.missingKeys.length + report.extraKeys.length
    statusBar.updateStatus(report.status, issueCount)
  }

  function notifyUser(
    report: SyncReport,
    errors: ParseError[],
    config: ReturnType<typeof readConfig>,
  ): void {
    if (config.notificationLevel === 'silent') {
      return
    }

    notifyMissingKeys(report)
    notifyParseErrors(errors)
  }

  function notifyMissingKeys(report: SyncReport): void {
    if (report.status !== 'missing-keys') {
      return
    }

    for (const mismatch of report.missingKeys) {
      notifier.showMissingKeys(mismatch.filepath, mismatch.keys)
    }
  }

  function notifyParseErrors(errors: ParseError[]): void {
    if (errors.length === 0) {
      return
    }

    const MAX_ERRORS_TO_SHOW = 3
    const errorsToShow = errors.slice(0, MAX_ERRORS_TO_SHOW)

    for (const error of errorsToShow) {
      const sanitizedMessage = sanitizeParseMessage(error.message)
      notifier.showParseError(error.filepath, sanitizedMessage)
    }
  }

  function trackTelemetry(report: SyncReport, files: DotenvFile[]): void {
    telemetry.event('sync-check', {
      status: report.status,
      fileCount: String(files.length),
      missingKeyCount: String(report.missingKeys.length),
    })
  }

  function handleSyncCheckError(error: unknown, config: ReturnType<typeof readConfig>): SyncReport {
    const errorReport = createErrorReport(error)

    statusBar.updateStatus('parse-error', 0)

    if (config.notificationLevel !== 'silent') {
      notifier.showError(`Failed to check dotenv sync: ${(error as Error).message}`)
    }

    return errorReport
  }

  function createDisabledReport(): SyncReport {
    return {
      status: 'no-files',
      files: Object.freeze([]),
      missingKeys: Object.freeze([]),
      extraKeys: Object.freeze([]),
      errors: Object.freeze([]),
      lastChecked: Date.now(),
    }
  }

  function createErrorReport(error: unknown): SyncReport {
    return {
      status: 'parse-error',
      files: Object.freeze([]),
      missingKeys: Object.freeze([]),
      extraKeys: Object.freeze([]),
      errors: Object.freeze([
        {
          type: 'read-error',
          message: `Failed to check sync: ${(error as Error).message}`,
          filepath: 'workspace',
        },
      ]),
      lastChecked: Date.now(),
    }
  }

  async function checkSyncForFiles(filePaths: readonly string[]): Promise<SyncReport> {
    try {
      return await performSyncCheckForFiles(filePaths)
    } catch (error) {
      return handleSyncCheckForFilesError(error)
    }
  }

  async function performSyncCheckForFiles(filePaths: readonly string[]): Promise<SyncReport> {
    const { files, errors } = await loadSpecificFiles(fileSystem, filePaths)
    const config = readConfig(configuration)

    const compareOptions = buildCompareOptions(files, config)
    const report = compareFiles(files, compareOptions)
    const finalReport = { ...report, errors: Object.freeze(errors) }

    updateUI(finalReport)
    notifyUser(finalReport, errors, config)
    trackSelectedFilesTelemetry(finalReport, files)

    return finalReport
  }

  function trackSelectedFilesTelemetry(report: SyncReport, files: DotenvFile[]): void {
    telemetry.event('sync-check-selected', {
      status: report.status,
      fileCount: String(files.length),
      missingKeyCount: String(report.missingKeys.length),
    })
  }

  function handleSyncCheckForFilesError(error: unknown): SyncReport {
    const errorReport = createSelectedFilesErrorReport(error)

    statusBar.updateStatus('parse-error', 0)

    const config = readConfig(configuration)
    if (config.notificationLevel !== 'silent') {
      notifier.showError(`Failed to check selected files: ${(error as Error).message}`)
    }

    return errorReport
  }

  function createSelectedFilesErrorReport(error: unknown): SyncReport {
    return {
      status: 'parse-error',
      files: Object.freeze([]),
      missingKeys: Object.freeze([]),
      extraKeys: Object.freeze([]),
      errors: Object.freeze([
        {
          type: 'read-error',
          message: `Failed to check selected files: ${(error as Error).message}`,
          filepath: 'selected-files',
        },
      ]),
      lastChecked: Date.now(),
    }
  }

  function dispose(): void {
    // Cleanup if needed
  }

  return Object.freeze({
    checkSync,
    checkSyncForFiles,
    dispose,
  })
}

async function discoverDotenvFiles(
  fileSystem: FileSystem,
  watchPatterns: readonly string[],
  excludePatterns: readonly string[],
  config: ReturnType<typeof readConfig>,
): Promise<{ files: DotenvFile[]; errors: ParseError[] }> {
  const files: DotenvFile[] = []
  const errors: ParseError[] = []

  const MAX_ERRORS = 50

  for (const pattern of watchPatterns) {
    if (hasExceededErrorLimit(errors, MAX_ERRORS)) {
      errors.push(createErrorLimitExceededError())
      break
    }

    await processPattern(pattern, fileSystem, excludePatterns, config, files, errors)
  }

  const filteredFiles = applyComparisonModeFilter(fileSystem, files, config)
  return { files: filteredFiles, errors }
}

function hasExceededErrorLimit(errors: ParseError[], maxErrors: number): boolean {
  return errors.length > maxErrors
}

function createErrorLimitExceededError(): ParseError {
  return {
    type: 'read-error',
    message: 'Too many parse errors detected. Check workspace configuration.',
    filepath: 'workspace',
  }
}

async function processPattern(
  pattern: string,
  fileSystem: FileSystem,
  excludePatterns: readonly string[],
  config: ReturnType<typeof readConfig>,
  files: DotenvFile[],
  errors: ParseError[],
): Promise<void> {
  try {
    const fileInfos = await fileSystem.findFiles(pattern, null, 100)
    await processFileInfos(fileInfos, fileSystem, excludePatterns, config, files, errors)
  } catch (error) {
    errors.push(createPatternSearchError(pattern, error))
  }
}

function createPatternSearchError(pattern: string, error: unknown): ParseError {
  return {
    type: 'read-error',
    message: `Failed to search pattern ${pattern}: ${(error as Error).message}`,
    filepath: 'pattern-search',
  }
}

async function processFileInfos(
  fileInfos: Array<{ filepath: string; uri: string }>,
  fileSystem: FileSystem,
  excludePatterns: readonly string[],
  config: ReturnType<typeof readConfig>,
  files: DotenvFile[],
  errors: ParseError[],
): Promise<void> {
  for (const info of fileInfos) {
    await processFileInfo(info, fileSystem, excludePatterns, config, files, errors)
  }
}

async function processFileInfo(
  info: { filepath: string; uri: string },
  fileSystem: FileSystem,
  excludePatterns: readonly string[],
  config: ReturnType<typeof readConfig>,
  files: DotenvFile[],
  errors: ParseError[],
): Promise<void> {
  const filepath = info.filepath
  const relativePath = fileSystem.asRelativePath(filepath)

  if (shouldSkipFile(relativePath, excludePatterns, config)) {
    return
  }

  await parseAndAddFile(filepath, fileSystem, files, errors)
}

function shouldSkipFile(
  relativePath: string,
  excludePatterns: readonly string[],
  config: ReturnType<typeof readConfig>,
): boolean {
  if (shouldExcludeFile(relativePath, excludePatterns)) {
    return true
  }

  if (isTemporarilyIgnored(relativePath, config)) {
    return true
  }

  return false
}

function isTemporarilyIgnored(
  relativePath: string,
  config: ReturnType<typeof readConfig>,
): boolean {
  return config.temporaryIgnore.includes(relativePath)
}

async function parseAndAddFile(
  filepath: string,
  fileSystem: FileSystem,
  files: DotenvFile[],
  errors: ParseError[],
): Promise<void> {
  try {
    const text = await fileSystem.readFile(filepath)
    const parseResult = parseDotenvFile(text, filepath)

    if (parseResult.errors.length > 0) {
      errors.push(...parseResult.errors)
    }

    if (parseResult.success) {
      const dotenvFile = await createDotenvFile(filepath, parseResult, fileSystem)
      files.push(dotenvFile)
    }
  } catch (error) {
    errors.push(createFileReadError(filepath, error))
  }
}

async function createDotenvFile(
  filepath: string,
  parseResult: ParseResult,
  fileSystem: FileSystem,
): Promise<DotenvFile> {
  const stat = await fileSystem.getFileStats(filepath)

  return {
    path: filepath,
    type: detectFileType(filepath),
    keys: parseResult.keys,
    lastModified: stat.mtime.getTime(),
  }
}

function createFileReadError(filepath: string, error: unknown): ParseError {
  return {
    type: 'read-error',
    message: (error as Error).message,
    filepath,
  }
}

async function loadSpecificFiles(
  fileSystem: FileSystem,
  filePaths: readonly string[],
): Promise<{ files: DotenvFile[]; errors: ParseError[] }> {
  const files: DotenvFile[] = []
  const errors: ParseError[] = []

  for (const filepath of filePaths) {
    await loadSpecificFile(filepath, fileSystem, files, errors)
  }

  return { files, errors }
}

async function loadSpecificFile(
  filepath: string,
  fileSystem: FileSystem,
  files: DotenvFile[],
  errors: ParseError[],
): Promise<void> {
  try {
    const text = await fileSystem.readFile(filepath)
    const parseResult = parseDotenvFile(text, filepath)

    if (parseResult.success) {
      const dotenvFile = await createDotenvFile(filepath, parseResult, fileSystem)
      files.push(dotenvFile)
    } else {
      errors.push(...parseResult.errors)
    }
  } catch (error) {
    errors.push(createFileReadError(filepath, error))
  }
}

function applyComparisonModeFilter(
  fileSystem: FileSystem,
  files: DotenvFile[],
  config: ReturnType<typeof readConfig>,
): DotenvFile[] {
  if (config.comparisonMode === 'manual') {
    return filterManualModeFiles(fileSystem, files, config)
  }

  return files
}

function filterManualModeFiles(
  fileSystem: FileSystem,
  files: DotenvFile[],
  config: ReturnType<typeof readConfig>,
): DotenvFile[] {
  if (config.compareOnlyFiles.length === 0) {
    return files
  }

  return files.filter((file) => isFileInCompareList(file, fileSystem, config))
}

function isFileInCompareList(
  file: DotenvFile,
  fileSystem: FileSystem,
  config: ReturnType<typeof readConfig>,
): boolean {
  const relativePath = fileSystem.asRelativePath(file.path)
  return config.compareOnlyFiles.includes(relativePath)
}

function sanitizeParseMessage(message: string): string {
  return message.replace(/^Failed to parse[^:]*:\s*/, '')
}
