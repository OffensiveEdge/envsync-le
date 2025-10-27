import type { DotenvFileType, ParseResult } from '../types'

const KEY_PATTERN = /^[a-zA-Z_][a-zA-Z0-9_-]*$/

export function parseDotenvFile(content: string, filepath: string): ParseResult {
  const keys: string[] = []
  const errors: Array<{
    type: 'parse-error' | 'read-error' | 'access-error'
    message: string
    filepath: string
  }> = []

  const lines = content.split('\n')

  for (let i = 0; i < lines.length; i++) {
    const rawLine = lines[i]
    if (rawLine === undefined) {
      continue
    }

    const line = rawLine.trim()
    const lineNumber = i + 1

    if (shouldSkipLine(line)) {
      continue
    }

    const parseError = parseEnvLine(line, lineNumber, filepath)
    if (parseError) {
      errors.push(parseError)
      continue
    }

    const key = extractKey(line)
    if (!key) {
      errors.push(createParseError(lineNumber, 'Empty key before equals sign', filepath))
      continue
    }

    if (!isValidKey(key)) {
      errors.push(createParseError(lineNumber, `Invalid key format "${key}"`, filepath))
      continue
    }

    keys.push(key)
  }

  return {
    success: true,
    keys: Object.freeze(keys),
    errors: Object.freeze(errors),
  }
}

function shouldSkipLine(line: string): boolean {
  if (!line) {
    return true
  }

  if (line.startsWith('#')) {
    return true
  }

  return false
}

function parseEnvLine(
  line: string,
  lineNumber: number,
  filepath: string,
): { type: 'parse-error'; message: string; filepath: string } | null {
  const equalIndex = line.indexOf('=')

  if (equalIndex === -1) {
    return createParseError(lineNumber, `Missing equals sign in "${line}"`, filepath)
  }

  return null
}

function extractKey(line: string): string {
  const equalIndex = line.indexOf('=')
  return line.substring(0, equalIndex).trim()
}

function isValidKey(key: string): boolean {
  return KEY_PATTERN.test(key)
}

function createParseError(
  lineNumber: number,
  message: string,
  filepath: string,
): { type: 'parse-error'; message: string; filepath: string } {
  return {
    type: 'parse-error',
    message: `Line ${lineNumber}: ${message}`,
    filepath,
  }
}

export function detectFileType(filepath: string): DotenvFileType {
  const filename = extractFilename(filepath)

  if (filename === '.env') {
    return 'base'
  }

  if (filename.includes('.local')) {
    return 'local'
  }

  if (isExampleFile(filename)) {
    return 'example'
  }

  if (isProductionFile(filename)) {
    return 'production'
  }

  if (isDevelopmentFile(filename)) {
    return 'development'
  }

  if (filename.includes('.test')) {
    return 'test'
  }

  return 'base'
}

function extractFilename(filepath: string): string {
  return filepath.split('/').pop() ?? ''
}

function isExampleFile(filename: string): boolean {
  return filename.includes('.example') || filename.includes('.template')
}

function isProductionFile(filename: string): boolean {
  return filename.includes('.production') || filename.includes('.prod')
}

function isDevelopmentFile(filename: string): boolean {
  return filename.includes('.development') || filename.includes('.dev')
}

export function shouldExcludeFile(filepath: string, excludePatterns: readonly string[]): boolean {
  return excludePatterns.some((pattern) => matchGlob(filepath, pattern))
}

function matchGlob(filepath: string, pattern: string): boolean {
  const regexPattern = convertGlobToRegex(pattern)
  return regexPattern.test(filepath)
}

function convertGlobToRegex(pattern: string): RegExp {
  const escaped = escapeRegexSpecialChars(pattern)
  const withDoubleStarMarker = replaceDoubleStarWithMarker(escaped)
  const withSingleStar = replaceSingleStarWithPattern(withDoubleStarMarker)
  const final = replaceMarkerWithDoubleStarPattern(withSingleStar)

  return new RegExp(`^${final}$`)
}

function escapeRegexSpecialChars(pattern: string): string {
  return pattern.replace(/[.+?^${}()|[\\]/g, '\\$&')
}

function replaceDoubleStarWithMarker(pattern: string): string {
  const DOUBLE_STAR_MARKER = '\u0000'
  return pattern.replace(/\*\*/g, DOUBLE_STAR_MARKER)
}

function replaceSingleStarWithPattern(pattern: string): string {
  return pattern.replace(/\*/g, '[^/]*')
}

function replaceMarkerWithDoubleStarPattern(pattern: string): string {
  const DOUBLE_STAR_MARKER = '\u0000'
  return pattern.replace(new RegExp(DOUBLE_STAR_MARKER, 'g'), '.*')
}
