import * as vscode from 'vscode';
import type {
	ProgressOptions,
	QuickPickItem,
	QuickPickOptions,
	UserInterface,
} from '../interfaces';

export function createVSCodeUserInterface(): UserInterface {
	return {
		async showProgress<T>(
			options: ProgressOptions,
			task: () => Promise<T>,
		): Promise<T> {
			const location =
				options.location === 'notification'
					? vscode.ProgressLocation.Notification
					: options.location === 'source-control'
						? vscode.ProgressLocation.SourceControl
						: vscode.ProgressLocation.Window;

			return await vscode.window.withProgress(
				{
					location,
					title: options.title,
					cancellable: options.cancellable ?? false,
				},
				task,
			);
		},

		async showQuickPick<T>(
			items: QuickPickItem<T>[],
			options: QuickPickOptions,
		): Promise<T | T[] | undefined> {
			const quickPickItems = items.map((item) => ({
				label: item.label,
				description: item.description ?? '',
				detail: item.detail ?? '',
			}));

			const vscodeOptions: vscode.QuickPickOptions = {
				canPickMany: options.canPickMany ?? false,
				ignoreFocusOut: options.ignoreFocusOut ?? false,
			};

			if (options.placeHolder) {
				vscodeOptions.placeHolder = options.placeHolder;
			}

			// With canPickMany the API resolves to an array; the static
			// overload types don't surface that for a non-literal flag.
			const selected = (await vscode.window.showQuickPick(
				quickPickItems,
				vscodeOptions,
			)) as (typeof quickPickItems)[number] | typeof quickPickItems | undefined;

			if (!selected) return undefined;

			const toValue = (pick: {
				label: string;
				description: string;
			}): T | undefined =>
				items.find(
					(item) =>
						item.label === pick.label &&
						(item.description ?? '') === pick.description,
				)?.value;

			if (Array.isArray(selected)) {
				return selected
					.map(toValue)
					.filter((value): value is T => value !== undefined);
			}

			return toValue(selected);
		},

		showInformationMessage(message: string): void {
			vscode.window.showInformationMessage(message);
		},

		showWarningMessage: ((message: string, ...actions: string[]) => {
			if (actions.length === 0) {
				vscode.window.showWarningMessage(message);
				return;
			}
			return Promise.resolve(
				vscode.window.showWarningMessage(message, ...actions),
			);
		}) as UserInterface['showWarningMessage'],

		showErrorMessage(message: string): void {
			vscode.window.showErrorMessage(message);
		},

		showStatusBarMessage(message: string, timeout?: number): void {
			if (timeout !== undefined) {
				vscode.window.setStatusBarMessage(message, timeout);
			} else {
				vscode.window.setStatusBarMessage(message);
			}
		},
	};
}
