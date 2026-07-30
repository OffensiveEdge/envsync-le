import { describe, expect, it } from 'vitest';
import {
	basename,
	detectFileType,
	isEnvFileName,
	shouldExcludeFile,
} from './heuristics';

describe('basename', () => {
	it('handles posix and windows separators', () => {
		expect(basename('/path/to/.env')).toBe('.env');
		expect(basename('C:\\project\\.env.local')).toBe('.env.local');
		expect(basename('.env')).toBe('.env');
	});
});

describe('isEnvFileName', () => {
	it('accepts .env, .env.*, and *.env', () => {
		expect(isEnvFileName('.env')).toBe(true);
		expect(isEnvFileName('.env.local')).toBe(true);
		expect(isEnvFileName('/a/b/.env.production')).toBe(true);
		expect(isEnvFileName('config.env')).toBe(true);
	});

	it('rejects non-env files including .envrc', () => {
		expect(isEnvFileName('.envrc')).toBe(false);
		expect(isEnvFileName('environment.ts')).toBe(false);
		expect(isEnvFileName('README.md')).toBe(false);
	});
});

describe('detectFileType', () => {
	it('should detect base .env files', () => {
		expect(detectFileType('.env')).toBe('base');
		expect(detectFileType('/path/to/.env')).toBe('base');
		expect(detectFileType('C:\\project\\.env')).toBe('base');
	});

	it('should detect local files', () => {
		expect(detectFileType('.env.local')).toBe('local');
		expect(detectFileType('.env.development.local')).toBe('local');
		expect(detectFileType('/path/to/.env.production.local')).toBe('local');
	});

	it('should detect example files', () => {
		expect(detectFileType('.env.example')).toBe('example');
		expect(detectFileType('.env.template')).toBe('example');
		expect(detectFileType('/path/to/.env.example')).toBe('example');
	});

	it('should detect production files', () => {
		expect(detectFileType('.env.production')).toBe('production');
		expect(detectFileType('.env.prod')).toBe('production');
		expect(detectFileType('/path/to/.env.production')).toBe('production');
	});

	it('should detect development files', () => {
		expect(detectFileType('.env.development')).toBe('development');
		expect(detectFileType('.env.dev')).toBe('development');
		expect(detectFileType('/path/to/.env.dev')).toBe('development');
	});

	it('should detect test files', () => {
		expect(detectFileType('.env.test')).toBe('test');
		expect(detectFileType('/path/to/.env.test')).toBe('test');
	});

	it('should fallback to base for unknown patterns', () => {
		expect(detectFileType('.env.unknown')).toBe('base');
		expect(detectFileType('random.txt')).toBe('base');
	});

	it('should classify by segment, not substring', () => {
		// 'app.device.env' contains '.dev' but is not a development env
		expect(detectFileType('app.device.env')).toBe('base');
		// suffix-style names are always base
		expect(detectFileType('foo.test.env')).toBe('base');
	});
});

describe('shouldExcludeFile', () => {
	it('should exclude files matching exact patterns', () => {
		const patterns = ['.env.local', '.env.production'];
		expect(shouldExcludeFile('.env.local', patterns)).toBe(true);
		expect(shouldExcludeFile('.env.production', patterns)).toBe(true);
		expect(shouldExcludeFile('.env.development', patterns)).toBe(false);
	});

	it('should handle glob patterns with asterisks', () => {
		const patterns = ['.env.*.local', '*.test'];
		expect(shouldExcludeFile('.env.development.local', patterns)).toBe(true);
		expect(shouldExcludeFile('.env.production.local', patterns)).toBe(true);
		expect(shouldExcludeFile('.env.test', patterns)).toBe(true);
		expect(shouldExcludeFile('.env.development', patterns)).toBe(false);
	});

	it('should match basenames for patterns without a slash', () => {
		// v1.x anchored the glob against the whole relative path, so
		// nested files never matched plain patterns
		const patterns = ['.env.*.local'];
		expect(shouldExcludeFile('config/.env.staging.local', patterns)).toBe(true);
		expect(shouldExcludeFile('config/.env.local', patterns)).toBe(false);
	});

	it('should handle empty exclude patterns', () => {
		expect(shouldExcludeFile('.env.local', [])).toBe(false);
	});

	it('should handle complex paths', () => {
		const patterns = ['**/.env.*.local'];
		expect(
			shouldExcludeFile('deep/nested/path/.env.development.local', patterns),
		).toBe(true);
		// '**/' also matches zero directories
		expect(shouldExcludeFile('.env.development.local', patterns)).toBe(true);
	});

	it('should match path-aware patterns', () => {
		const patterns = ['packages/**/.env.*.local'];
		expect(
			shouldExcludeFile('packages/app/.env.production.local', patterns),
		).toBe(true);
		expect(
			shouldExcludeFile('packages/web/nested/.env.dev.local', patterns),
		).toBe(true);
		expect(shouldExcludeFile('other/.env.dev.local', patterns)).toBe(false);
	});

	it('should treat windows separators as path separators', () => {
		expect(
			shouldExcludeFile('config\\.env.staging.local', ['.env.*.local']),
		).toBe(true);
	});
});
