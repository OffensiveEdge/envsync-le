import type * as vscode from 'vscode';
import type { Detector } from '../detection/detector';
import type { Configuration, FileSystem, UserInterface } from '../interfaces';
import type { Telemetry } from '../interfaces/telemetry';
import { registerCompareSelectedCommand } from './compareSelected';
import { registerHelpCommand } from './help';
import { registerIgnoreFileCommand } from './ignoreFile';
import { registerSetTemplateCommand } from './setTemplate';
import { registerShowIssuesCommand } from './showIssues';

// Centralized command registration. The deps bag carries exactly what
// the commands consume — no notifier/statusBar (nothing here uses them).
export function registerAllCommands(
	context: vscode.ExtensionContext,
	deps: Readonly<{
		telemetry: Telemetry;
		detector: Detector;
		fileSystem: FileSystem;
		ui: UserInterface;
		configuration: Configuration;
	}>,
): void {
	registerCompareSelectedCommand(context, deps);
	registerSetTemplateCommand(context, deps);
	registerIgnoreFileCommand(context, deps);
	registerShowIssuesCommand(context, deps);
	registerHelpCommand(context, deps);
}
