import type { DotenvFile, KeyMismatch, SyncReport, SyncStatus } from '../types';

interface CompareOptions {
	mode?: 'auto' | 'template';
	templatePath?: string;
	caseSensitive?: boolean;
}

export function compareFiles(
	files: readonly DotenvFile[],
	options?: CompareOptions,
): SyncReport {
	if (files.length === 0) {
		return createEmptyReport();
	}

	const caseSensitive = options?.caseSensitive ?? true;
	const referenceKeys = collectReferenceKeys(files, options);
	const missingKeys = findMissingKeys(files, referenceKeys, caseSensitive);

	// Extra keys only exist relative to a template: in auto mode the
	// reference is the union of all keys, so nothing can be "extra".
	const template = shouldUseTemplateMode(options)
		? findTemplateFile(files, options?.templatePath ?? '')
		: undefined;
	const extraKeys = template
		? findExtraKeys(files, referenceKeys, caseSensitive, template)
		: [];

	const status = determineStatus(missingKeys, extraKeys);

	return {
		status,
		files: Object.freeze([...files]),
		missingKeys: Object.freeze(missingKeys),
		extraKeys: Object.freeze(extraKeys),
		errors: Object.freeze([]),
		lastChecked: Date.now(),
	};
}

function createEmptyReport(): SyncReport {
	return {
		status: 'no-files',
		files: Object.freeze([]),
		missingKeys: Object.freeze([]),
		extraKeys: Object.freeze([]),
		errors: Object.freeze([]),
		lastChecked: Date.now(),
	};
}

function collectReferenceKeys(
	files: readonly DotenvFile[],
	options?: CompareOptions,
): Set<string> {
	if (shouldUseTemplateMode(options) && options?.templatePath) {
		return collectTemplateKeys(files, options.templatePath);
	}

	return collectAllKeys(files);
}

function shouldUseTemplateMode(options?: CompareOptions): boolean {
	return options?.mode === 'template' && Boolean(options.templatePath);
}

function collectTemplateKeys(
	files: readonly DotenvFile[],
	templatePath: string | undefined,
): Set<string> {
	if (!templatePath) {
		return collectAllKeys(files);
	}

	const template = findTemplateFile(files, templatePath);

	if (!template) {
		return collectAllKeys(files);
	}

	return new Set(template.keys);
}

function findTemplateFile(
	files: readonly DotenvFile[],
	templatePath: string,
): DotenvFile | undefined {
	return files.find((file) => file.path === templatePath);
}

function collectAllKeys(files: readonly DotenvFile[]): Set<string> {
	const allKeys = new Set<string>();

	for (const file of files) {
		for (const key of file.keys) {
			allKeys.add(key);
		}
	}

	return allKeys;
}

function normalizeKey(key: string, caseSensitive: boolean): string {
	return caseSensitive ? key : key.toLowerCase();
}

function findMissingKeys(
	files: readonly DotenvFile[],
	referenceKeys: Set<string>,
	caseSensitive: boolean,
): KeyMismatch[] {
	const mismatches: KeyMismatch[] = [];

	for (const file of files) {
		const missing = findMissingKeysInFile(file, referenceKeys, caseSensitive);

		if (missing.length === 0) {
			continue;
		}

		const reference = findReferenceFile(files, file, missing, caseSensitive);

		mismatches.push({
			filepath: file.path,
			keys: Object.freeze([...missing]),
			reference: reference?.path ?? 'other files',
		});
	}

	return mismatches;
}

function findMissingKeysInFile(
	file: DotenvFile,
	referenceKeys: Set<string>,
	caseSensitive: boolean,
): string[] {
	const fileKeys = new Set(
		file.keys.map((key) => normalizeKey(key, caseSensitive)),
	);
	const missing: string[] = [];

	for (const key of referenceKeys) {
		if (!fileKeys.has(normalizeKey(key, caseSensitive))) {
			missing.push(key);
		}
	}

	return missing;
}

function findReferenceFile(
	files: readonly DotenvFile[],
	currentFile: DotenvFile,
	missingKeys: string[],
	caseSensitive: boolean,
): DotenvFile | undefined {
	return files.find(
		(file) =>
			file.path !== currentFile.path &&
			hasAllKeys(file, missingKeys, caseSensitive),
	);
}

function hasAllKeys(
	file: DotenvFile,
	keys: string[],
	caseSensitive: boolean,
): boolean {
	const fileKeys = new Set(
		file.keys.map((key) => normalizeKey(key, caseSensitive)),
	);
	return keys.every((key) => fileKeys.has(normalizeKey(key, caseSensitive)));
}

function findExtraKeys(
	files: readonly DotenvFile[],
	referenceKeys: Set<string>,
	caseSensitive: boolean,
	template: DotenvFile,
): KeyMismatch[] {
	const normalizedReference = new Set(
		[...referenceKeys].map((key) => normalizeKey(key, caseSensitive)),
	);
	const mismatches: KeyMismatch[] = [];

	for (const file of files) {
		if (file.path === template.path) {
			continue;
		}

		const extra = file.keys.filter(
			(key) => !normalizedReference.has(normalizeKey(key, caseSensitive)),
		);

		if (extra.length > 0) {
			mismatches.push({
				filepath: file.path,
				keys: Object.freeze([...extra]),
				reference: template.path,
			});
		}
	}

	return mismatches;
}

function determineStatus(
	missingKeys: KeyMismatch[],
	extraKeys: KeyMismatch[],
): SyncStatus {
	if (missingKeys.length > 0) {
		return 'missing-keys';
	}

	if (extraKeys.length > 0) {
		return 'extra-keys';
	}

	return 'in-sync';
}

export function areFilesInSync(files: readonly DotenvFile[]): boolean {
	if (files.length <= 1) {
		return true;
	}

	const firstFile = files[0];
	if (!firstFile) {
		return true;
	}

	const referenceKeys = new Set(firstFile.keys);
	const remainingFiles = files.slice(1);

	return remainingFiles.every((file) => hasMatchingKeys(file, referenceKeys));
}

function hasMatchingKeys(
	file: DotenvFile,
	referenceKeys: Set<string>,
): boolean {
	const fileKeys = new Set(file.keys);

	if (fileKeys.size !== referenceKeys.size) {
		return false;
	}

	return hasAllKeysFromSet(fileKeys, referenceKeys);
}

function hasAllKeysFromSet(
	fileKeys: Set<string>,
	referenceKeys: Set<string>,
): boolean {
	for (const key of referenceKeys) {
		if (!fileKeys.has(key)) {
			return false;
		}
	}

	return true;
}
