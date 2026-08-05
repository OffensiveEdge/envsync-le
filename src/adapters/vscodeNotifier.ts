import * as vscode from 'vscode';
import type { readConfig } from '../config/config';
import { basename } from '../detection/heuristics';
import type { Configuration } from '../interfaces';
import type { Notifier } from '../interfaces/notifier';

interface VSCodeDependencies {
	window: typeof vscode.window;
	readConfig: typeof readConfig;
}

export function createVSCodeNotifier(
	deps: VSCodeDependencies,
	configuration: Configuration,
): Notifier {
	function getNotificationLevel(): 'all' | 'important' | 'silent' {
		const config = deps.readConfig(configuration);
		return config.notificationLevel;
	}

	function showMissingKeys(filepath: string, keys: readonly string[]): void {
		const level = getNotificationLevel();

		if (shouldSuppressNotification(level)) {
			return;
		}

		if (!shouldShowImportantNotification(level)) {
			return;
		}

		const message = formatMissingKeysMessage(filepath, keys);
		deps.window.showWarningMessage(message);
	}

	function showExtraKeys(filepath: string, keys: readonly string[]): void {
		const level = getNotificationLevel();

		if (shouldSuppressNotification(level)) {
			return;
		}

		if (!shouldShowAllNotifications(level)) {
			return;
		}

		const message = formatExtraKeysMessage(filepath, keys);
		deps.window.showInformationMessage(message);
	}

	function showError(message: string): void {
		const level = getNotificationLevel();

		if (shouldSuppressNotification(level)) {
			return;
		}

		deps.window.showErrorMessage(message);
	}

	function showParseError(filepath: string, error: string): void {
		const level = getNotificationLevel();

		if (shouldSuppressNotification(level)) {
			return;
		}

		if (!shouldShowImportantNotification(level)) {
			return;
		}

		const message = formatParseErrorMessage(filepath, error);
		deps.window.showErrorMessage(message);
	}

	return Object.freeze({
		showMissingKeys,
		showExtraKeys,
		showError,
		showParseError,
	});
}

function shouldSuppressNotification(level: string): boolean {
	return level === 'silent';
}

function shouldShowAllNotifications(level: string): boolean {
	return level === 'all';
}

function shouldShowImportantNotification(level: string): boolean {
	return level === 'all' || level === 'important';
}

function formatMissingKeysMessage(
	filepath: string,
	keys: readonly string[],
): string {
	const filename = extractFilename(filepath);
	const keyList = formatKeyList(keys);

	return vscode.l10n.t('Missing keys in {0}: {1}', filename, keyList);
}

function formatExtraKeysMessage(
	filepath: string,
	keys: readonly string[],
): string {
	const filename = extractFilename(filepath);
	const keyList = formatKeyList(keys);

	return vscode.l10n.t('Extra keys in {0}: {1}', filename, keyList);
}

function formatParseErrorMessage(filepath: string, error: string): string {
	const filename = extractFilename(filepath);

	return vscode.l10n.t('Failed to parse {0}: {1}', filename, error);
}

function extractFilename(filepath: string): string {
	return basename(filepath) || filepath;
}

function formatKeyList(keys: readonly string[]): string {
	const MAX_KEYS_TO_SHOW = 3;
	const visibleKeys = keys.slice(0, MAX_KEYS_TO_SHOW);
	const hasMore = keys.length > MAX_KEYS_TO_SHOW;

	return visibleKeys.join(', ') + (hasMore ? '...' : '');
}
