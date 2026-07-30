import type { DotenvFileType } from '../types';

/**
 * Shared filename/path heuristics for dotenv detection. This is the one
 * place that decides what counts as a dotenv file, how a file is
 * classified, and how exclude globs match — previously three divergent
 * copies (parser, watcher, and per-command string slicing).
 *
 * Intentional behavior (documented, not bugs):
 * - Classification is segment-based on the basename: `.env.production`
 *   is production, but `app.device.env` is NOT development — substring
 *   matching used to misclassify it. Suffix-style names (`foo.env`) are
 *   always 'base'.
 * - Priority when multiple segments match: local > example > production
 *   > development > test (`.env.production.local` is a local override).
 * - Exclude patterns without a '/' match against the basename
 *   (gitignore-style), so '.env.*.local' excludes nested files too.
 *   Patterns with a '/' match against the workspace-relative path;
 *   a leading '**' + '/' matches zero or more directories.
 */

export function basename(filepath: string): string {
	const normalized = filepath.replace(/\\/g, '/');
	return normalized.split('/').pop() ?? '';
}

export function isEnvFileName(filepath: string): boolean {
	const name = basename(filepath);
	return name === '.env' || name.startsWith('.env.') || name.endsWith('.env');
}

export function detectFileType(filepath: string): DotenvFileType {
	const name = basename(filepath);

	if (name !== '.env' && !name.startsWith('.env.')) {
		return 'base'; // suffix-style names (foo.env) and everything else
	}

	// '.env.development.local' -> ['development', 'local']
	const segments = new Set(name.split('.').slice(2));

	if (segments.has('local')) return 'local';
	if (segments.has('example') || segments.has('template')) return 'example';
	if (segments.has('production') || segments.has('prod')) return 'production';
	if (segments.has('development') || segments.has('dev')) return 'development';
	if (segments.has('test')) return 'test';

	return 'base';
}

export function shouldExcludeFile(
	filepath: string,
	excludePatterns: readonly string[],
): boolean {
	const path = filepath.replace(/\\/g, '/');
	const name = basename(path);

	return excludePatterns.some((pattern) => {
		const target = pattern.includes('/') ? path : name;
		return globToRegex(pattern).test(target);
	});
}

function globToRegex(pattern: string): RegExp {
	let out = '';
	let i = 0;

	while (i < pattern.length) {
		const c = pattern[i] as string;

		if (c === '*') {
			if (pattern[i + 1] === '*') {
				if (pattern[i + 2] === '/') {
					out += '(?:.*/)?'; // '**/' spans zero or more directories
					i += 3;
				} else {
					out += '.*';
					i += 2;
				}
			} else {
				out += '[^/]*';
				i += 1;
			}
		} else if (c === '?') {
			out += '[^/]';
			i += 1;
		} else {
			out += escapeRegexChar(c);
			i += 1;
		}
	}

	return new RegExp(`^${out}$`);
}

function escapeRegexChar(c: string): string {
	return /[.+^${}()|[\]\\]/.test(c) ? `\\${c}` : c;
}
