import * as assert from 'node:assert';
import * as vscode from 'vscode';

const EXTENSION_ID = 'nolindnaidoo.envsync-le';

describe('EnvSync-LE integration', function () {
	this.timeout(30_000);

	it('activates', async () => {
		const extension = vscode.extensions.getExtension(EXTENSION_ID);
		assert.ok(extension, `extension ${EXTENSION_ID} not found`);
		await extension.activate();
		assert.strictEqual(extension.isActive, true);
	});

	it('registers every declared command', async () => {
		const extension = vscode.extensions.getExtension(EXTENSION_ID);
		await extension?.activate();
		const commands = await vscode.commands.getCommands(true);
		for (const id of [
			'envsync-le.openSettings',
			'envsync-le.showIssues',
			'envsync-le.compareSelected',
			'envsync-le.setTemplate',
			'envsync-le.clearTemplate',
			'envsync-le.ignoreFile',
			'envsync-le.stopIgnoring',
			'envsync-le.clearAllIgnored',
			'envsync-le.help',
		]) {
			assert.ok(commands.includes(id), `missing command: ${id}`);
		}
	});

	it('showIssues reports the missing key across the fixture files', async () => {
		// fixtures: .env has DATABASE_URL, API_KEY, DEBUG; .env.local
		// lacks DEBUG — the report must call that out.
		await vscode.commands.executeCommand('envsync-le.showIssues');

		const reportDoc = vscode.workspace.textDocuments.find(
			(doc) =>
				doc.languageId === 'markdown' &&
				doc.getText().includes('Sync Report'),
		);
		assert.ok(reportDoc, 'no sync report document found');
		const text = reportDoc.getText();
		assert.ok(text.includes('Missing Keys'), 'report lacks Missing Keys section');
		assert.ok(text.includes('.env.local'), 'report does not name .env.local');
		assert.ok(text.includes('- DEBUG'), 'report does not list the missing key');
	});

	it('help opens a markdown document', async () => {
		await vscode.commands.executeCommand('envsync-le.help');

		const helpDoc = vscode.workspace.textDocuments.find((doc) =>
			doc.getText().includes('# EnvSync-LE Help'),
		);
		assert.ok(helpDoc, 'no help document found');
		assert.strictEqual(helpDoc.languageId, 'markdown');
	});
});
