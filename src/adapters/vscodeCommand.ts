import * as vscode from 'vscode';
import type { CommandAdapter } from '../interfaces/command';

/**
 * The VS Code command surface, as a frozen object.
 *
 * Was the fleet's only class. Nothing inherited from it and nothing needed an
 * instance identity — it held a context and closed over it, which a factory
 * does without `this`.
 */
export function createVSCodeCommandAdapter(
	context: vscode.ExtensionContext,
): CommandAdapter {
	return Object.freeze({
		registerCommand(
			command: string,
			callback: (...args: unknown[]) => unknown | undefined,
		): void {
			context.subscriptions.push(
				vscode.commands.registerCommand(command, (...args: unknown[]) =>
					callback(...args),
				),
			);
		},

		executeCommand<T = unknown>(
			command: string,
			...rest: unknown[]
		): Promise<T | undefined> {
			const exec = vscode.commands.executeCommand as <R = unknown>(
				cmd: string,
				...args: unknown[]
			) => Thenable<R | undefined>;
			return Promise.resolve(exec<T>(command, ...rest));
		},
	});
}
