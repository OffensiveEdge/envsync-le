import * as vscode from 'vscode'
import { readConfig } from '../config/config'
import type { Detector } from '../detection/detector'
import type { Configuration, FileSystem } from '../interfaces'

export function registerVSCodeWatchers(
	context: vscode.ExtensionContext,
	detector: Detector,
	configuration: Configuration,
	fileSystem: FileSystem,
): void {
	const config = readConfig(configuration)

	// Debounced detection shared by all watchers
	let timeoutId: NodeJS.Timeout | undefined
	let isChecking = false
	const debouncedDetect = (): void => {
		if (timeoutId) clearTimeout(timeoutId)
		timeoutId = setTimeout(async () => {
			if (isChecking) return // Prevent concurrent checks
			isChecking = true
			try {
				await detector.checkSync()
			} catch (error) {
				// Only log if notifications are enabled - respect user's preference
				if (config.notificationLevel !== 'silent') {
					console.error('File watcher sync check failed:', error)
				}
			} finally {
				isChecking = false
			}
		}, config.debounceMs)
	}

	const watchers: vscode.FileSystemWatcher[] = []

	for (const pattern of config.watchPatterns) {
		const watcher = vscode.workspace.createFileSystemWatcher(pattern)

		const handleEvent = async (uri: vscode.Uri): Promise<void> => {
			try {
				// Apply exclude patterns on relative path
				const rel = fileSystem.asRelativePath(uri.fsPath)
				const excluded = config.excludePatterns.some((pat) => matchSimpleGlob(rel, pat))
				if (excluded) return
				debouncedDetect()
			} catch (error) {
				// Log but don't crash the watcher
				if (config.notificationLevel !== 'silent') {
					console.error('File watcher event error:', error)
				}
			}
		}

		watcher.onDidCreate(handleEvent)
		watcher.onDidChange(handleEvent)
		watcher.onDidDelete(handleEvent)

		context.subscriptions.push(watcher)
		watchers.push(watcher)
	}

	// Cleanup debounce timer
	context.subscriptions.push({
		dispose: () => timeoutId && clearTimeout(timeoutId),
	})
}

// Very small glob matcher: supports "**", "*" and simple path segments
function matchSimpleGlob(input: string, pattern: string): boolean {
	// Escape regex special chars except *
	const escaped = pattern.replace(/[.+?^${}()|[\\]/g, '\\$&')
	// Use a placeholder to avoid interfering replacements
	const DOUBLE_STAR = '\u0000'
	const withMarker = escaped.replace(/\*\*/g, DOUBLE_STAR)
	const singleExpanded = withMarker.replace(/\*/g, '[^/]*')
	const regexStr = `^${singleExpanded.replace(new RegExp(DOUBLE_STAR, 'g'), '.*')}$`
	return new RegExp(regexStr).test(input)
}
