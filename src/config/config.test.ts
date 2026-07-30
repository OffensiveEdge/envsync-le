import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { describe, expect, it } from 'vitest';
import type { Configuration } from '../interfaces';
import { CONFIG_DEFAULTS, readConfig } from './config';

function makeConfig(values: Record<string, unknown> = {}): Configuration {
	return {
		get: <T>(key: string, defaultValue: T): T =>
			key in values ? (values[key] as T) : defaultValue,
		has: (key: string) => key in values,
	};
}

describe('readConfig', () => {
	it('should return default values when configuration is empty', () => {
		const result = readConfig(makeConfig());

		expect(result.enabled).toBe(true);
		expect(result.watchPatterns).toEqual(['.env*']);
		expect(result.excludePatterns).toEqual([]);
		expect(result.notificationLevel).toBe('important');
		expect(result.statusBarEnabled).toBe(true);
		expect(result.debounceMs).toBe(1000);
		expect(result.caseSensitive).toBe(true);
		expect(result.telemetryEnabled).toBe(false);
		expect(result.comparisonMode).toBe('auto');
		expect(result.compareOnlyFiles).toEqual([]);
		expect(result.templateFile).toBeUndefined();
		expect(result.temporaryIgnore).toEqual([]);
	});

	it('should handle custom configuration values', () => {
		const result = readConfig(
			makeConfig({
				enabled: false,
				watchPatterns: ['.env', '.env.local'],
				excludePatterns: ['.env.prod'],
				notificationLevel: 'all',
				'statusBar.enabled': false,
				debounceMs: 500,
				caseSensitive: false,
				telemetryEnabled: true,
				comparisonMode: 'manual',
				compareOnlyFiles: ['.env.template'],
				templateFile: '.env.template',
				temporaryIgnore: ['.env.ignored'],
			}),
		);

		expect(result.enabled).toBe(false);
		expect(result.watchPatterns).toEqual(['.env', '.env.local']);
		expect(result.excludePatterns).toEqual(['.env.prod']);
		expect(result.notificationLevel).toBe('all');
		expect(result.statusBarEnabled).toBe(false);
		expect(result.debounceMs).toBe(500);
		expect(result.caseSensitive).toBe(false);
		expect(result.telemetryEnabled).toBe(true);
		expect(result.comparisonMode).toBe('manual');
		expect(result.compareOnlyFiles).toEqual(['.env.template']);
		expect(result.templateFile).toBe('.env.template');
		expect(result.temporaryIgnore).toEqual(['.env.ignored']);
	});

	it('should enforce minimum debounce value', () => {
		const result = readConfig(makeConfig({ debounceMs: 50 }));
		expect(result.debounceMs).toBe(100); // Clamped to minimum
	});

	it('should fall back to the default on a non-numeric debounce', () => {
		const result = readConfig(makeConfig({ debounceMs: 'soon' }));
		expect(result.debounceMs).toBe(1000); // NaN must not survive Math.max
	});

	it('should ignore the never-declared notificationsLevel typo key', () => {
		const result = readConfig(makeConfig({ notificationsLevel: 'all' }));
		expect(result.notificationLevel).toBe('important');
	});

	it('should validate notification levels', () => {
		const result = readConfig(makeConfig({ notificationLevel: 'loud' }));
		expect(result.notificationLevel).toBe('important');
	});

	it('should validate comparison modes', () => {
		const result = readConfig(makeConfig({ comparisonMode: 'invalid-mode' }));
		expect(result.comparisonMode).toBe('auto');
	});

	it('should not coerce non-boolean values to true', () => {
		const result = readConfig(makeConfig({ telemetryEnabled: 'yes' }));
		expect(result.telemetryEnabled).toBe(false);
	});

	it('should reject non-array pattern values', () => {
		const result = readConfig(makeConfig({ watchPatterns: '.env*' }));
		expect(result.watchPatterns).toEqual(['.env*']);
	});

	it('should treat an empty templateFile as unset', () => {
		const result = readConfig(makeConfig({ templateFile: '' }));
		expect(result.templateFile).toBeUndefined();
	});

	it('should return frozen arrays for immutability', () => {
		const result = readConfig(
			makeConfig({
				watchPatterns: ['.env*'],
				excludePatterns: ['.env.*.local'],
				compareOnlyFiles: [],
				temporaryIgnore: [],
			}),
		);

		expect(Object.isFrozen(result.watchPatterns)).toBe(true);
		expect(Object.isFrozen(result.excludePatterns)).toBe(true);
		expect(Object.isFrozen(result.compareOnlyFiles)).toBe(true);
		expect(Object.isFrozen(result.temporaryIgnore)).toBe(true);
	});
});

/**
 * CONFIG_DEFAULTS must stay identical to the defaults declared in
 * package.json contributes.configuration — v1.x shipped with the two
 * silently disagreeing (excludePatterns defaulted to [] in the manifest
 * but ['.env.*.local'] in code).
 */
describe('config defaults parity with package.json', () => {
	const manifest = JSON.parse(
		readFileSync(join(__dirname, '..', '..', 'package.json'), 'utf8'),
	) as {
		contributes: {
			configuration: { properties: Record<string, { default?: unknown }> };
		};
	};
	const props = manifest.contributes.configuration.properties;

	const KEY_MAP: Record<string, keyof typeof CONFIG_DEFAULTS> = {
		'envsync-le.enabled': 'enabled',
		'envsync-le.watchPatterns': 'watchPatterns',
		'envsync-le.excludePatterns': 'excludePatterns',
		'envsync-le.notificationLevel': 'notificationLevel',
		'envsync-le.statusBar.enabled': 'statusBarEnabled',
		'envsync-le.debounceMs': 'debounceMs',
		'envsync-le.caseSensitive': 'caseSensitive',
		'envsync-le.telemetryEnabled': 'telemetryEnabled',
		'envsync-le.comparisonMode': 'comparisonMode',
		'envsync-le.compareOnlyFiles': 'compareOnlyFiles',
		'envsync-le.templateFile': 'templateFile',
		'envsync-le.temporaryIgnore': 'temporaryIgnore',
	};

	it('covers every declared setting', () => {
		expect(Object.keys(props).sort()).toEqual(Object.keys(KEY_MAP).sort());
	});

	for (const [manifestKey, defaultsKey] of Object.entries(KEY_MAP)) {
		it(`${manifestKey} default matches`, () => {
			expect(CONFIG_DEFAULTS[defaultsKey]).toEqual(props[manifestKey]?.default);
		});
	}
});
