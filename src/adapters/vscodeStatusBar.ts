import type * as vscode from 'vscode';
import type { readConfig } from '../config/config';
import type { Configuration } from '../interfaces';
import type { StatusBar } from '../interfaces/statusBar';
import type { SyncStatus } from '../types';

interface VSCodeDependencies {
	window: typeof vscode.window; // Modified
	StatusBarAlignment: typeof vscode.StatusBarAlignment; // Modified
	ThemeColor: typeof vscode.ThemeColor; // Modified
	readConfig: typeof readConfig;
}

export function createVSCodeStatusBar(
	context: vscode.ExtensionContext, // Modified
	deps: VSCodeDependencies,
	configuration?: Configuration,
): StatusBar {
	const statusBarItem = deps.window.createStatusBarItem(
		deps.StatusBarAlignment.Left,
		100,
	);
	context.subscriptions.push(statusBarItem);

	function updateStatus(status: SyncStatus, issueCount: number): void {
		const config = deps.readConfig(
			configuration ?? createDefaultConfiguration(),
		);

		if (!config.statusBarEnabled) {
			statusBarItem.hide();
			return;
		}

		if (shouldHideStatusBar(status)) {
			statusBarItem.hide();
			return;
		}

		applyStatusConfiguration(statusBarItem, status, issueCount, deps);
		statusBarItem.show();
	}

	function dispose(): void {
		statusBarItem.dispose();
	}

	return Object.freeze({
		updateStatus,
		dispose,
	});
}

function createDefaultConfiguration(): Configuration {
	return {
		get: <T>(_k: string, d: T) => d,
		getSection: () => ({
			get: <T>(_k: string, d: T) => d,
			getSection: () => ({}) as never,
			has: () => false,
		}),
		has: () => false,
	};
}

function shouldHideStatusBar(status: SyncStatus): boolean {
	return status === 'no-files';
}

function applyStatusConfiguration(
	statusBarItem: vscode.StatusBarItem,
	status: SyncStatus,
	issueCount: number,
	deps: VSCodeDependencies,
): void {
	if (status === 'in-sync') {
		configureInSyncStatus(statusBarItem);
		return;
	}

	if (status === 'missing-keys' || status === 'extra-keys') {
		configureOutOfSyncStatus(statusBarItem, issueCount, deps);
		return;
	}

	if (status === 'parse-error') {
		configureErrorStatus(statusBarItem, issueCount, deps);
		return;
	}
}

function configureInSyncStatus(statusBarItem: vscode.StatusBarItem): void {
	statusBarItem.text = '$(file) 0';
	statusBarItem.tooltip = 'All dotenv files are in sync';
	statusBarItem.backgroundColor = undefined;
	statusBarItem.command = undefined;
}

function configureOutOfSyncStatus(
	statusBarItem: vscode.StatusBarItem,
	issueCount: number,
	deps: VSCodeDependencies,
): void {
	statusBarItem.text = `$(file) ${issueCount}`;
	statusBarItem.tooltip = 'Dotenv files out of sync - click for details';
	statusBarItem.backgroundColor = new deps.ThemeColor(
		'statusBarItem.warningBackground',
	);
	statusBarItem.command = 'envsync-le.showIssues';
}

function configureErrorStatus(
	statusBarItem: vscode.StatusBarItem,
	issueCount: number,
	deps: VSCodeDependencies,
): void {
	const displayCount = issueCount > 0 ? issueCount : '';

	statusBarItem.text = `$(file) ${displayCount}`;
	statusBarItem.tooltip = 'Error checking dotenv files - click for settings';
	statusBarItem.backgroundColor = new deps.ThemeColor(
		'statusBarItem.errorBackground',
	);
	statusBarItem.command = 'envsync-le.showIssues';
}
