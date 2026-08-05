import type { Configuration } from '../interfaces';
import type {
	ComparisonMode,
	DotSyncConfig,
	NotificationLevel,
} from '../types';

/**
 * The defaults, exported for the parity gate.
 *
 * Nothing else imports this: `config.test.ts` asserts it matches every
 * default declared in package.json, which is the invariant that stops the
 * two drifting apart. The export is the seam that test needs.
 */
export const CONFIG_DEFAULTS = Object.freeze({
	enabled: true,
	watchPatterns: Object.freeze(['.env*']) as readonly string[],
	excludePatterns: Object.freeze([]) as readonly string[],
	notificationLevel: 'important' as NotificationLevel,
	statusBarEnabled: true,
	debounceMs: 1000,
	caseSensitive: true,
	telemetryEnabled: false,
	comparisonMode: 'auto' as ComparisonMode,
	compareOnlyFiles: Object.freeze([]) as readonly string[],
	templateFile: undefined as string | undefined,
	temporaryIgnore: Object.freeze([]) as readonly string[],
});

export function readConfig(configuration: Configuration): DotSyncConfig {
	return Object.freeze({
		enabled: readBoolean(configuration, 'enabled', CONFIG_DEFAULTS.enabled),
		watchPatterns: readStringArray(
			configuration,
			'watchPatterns',
			CONFIG_DEFAULTS.watchPatterns,
		),
		excludePatterns: readStringArray(
			configuration,
			'excludePatterns',
			CONFIG_DEFAULTS.excludePatterns,
		),
		notificationLevel: readNotificationLevel(configuration),
		statusBarEnabled: readBoolean(
			configuration,
			'statusBar.enabled',
			CONFIG_DEFAULTS.statusBarEnabled,
		),
		debounceMs: readNumber(
			configuration,
			'debounceMs',
			CONFIG_DEFAULTS.debounceMs,
			100,
		),
		caseSensitive: readBoolean(
			configuration,
			'caseSensitive',
			CONFIG_DEFAULTS.caseSensitive,
		),
		telemetryEnabled: readBoolean(
			configuration,
			'telemetryEnabled',
			CONFIG_DEFAULTS.telemetryEnabled,
		),
		comparisonMode: readComparisonMode(configuration),
		compareOnlyFiles: readStringArray(
			configuration,
			'compareOnlyFiles',
			CONFIG_DEFAULTS.compareOnlyFiles,
		),
		templateFile: readTemplateFile(configuration),
		temporaryIgnore: readStringArray(
			configuration,
			'temporaryIgnore',
			CONFIG_DEFAULTS.temporaryIgnore,
		),
	});
}

function readBoolean(
	configuration: Configuration,
	key: string,
	defaultValue: boolean,
): boolean {
	const value = configuration.get<boolean>(key, defaultValue);
	return typeof value === 'boolean' ? value : defaultValue;
}

function readNumber(
	configuration: Configuration,
	key: string,
	defaultValue: number,
	minValue: number,
): number {
	const value = Number(configuration.get<number>(key, defaultValue));
	if (!Number.isFinite(value)) {
		return defaultValue;
	}
	return Math.max(minValue, value);
}

function readStringArray(
	configuration: Configuration,
	key: string,
	defaultValue: readonly string[],
): readonly string[] {
	const value = configuration.get<readonly string[]>(key, defaultValue);
	if (!Array.isArray(value)) {
		return defaultValue;
	}
	return Object.freeze(value.filter((v): v is string => typeof v === 'string'));
}

function readNotificationLevel(
	configuration: Configuration,
): NotificationLevel {
	const raw = configuration.get<string>(
		'notificationLevel',
		CONFIG_DEFAULTS.notificationLevel,
	);
	return isValidNotificationLevel(raw)
		? raw
		: CONFIG_DEFAULTS.notificationLevel;
}

function readComparisonMode(configuration: Configuration): ComparisonMode {
	const raw = configuration.get<string>(
		'comparisonMode',
		CONFIG_DEFAULTS.comparisonMode,
	);
	return isValidComparisonMode(raw) ? raw : CONFIG_DEFAULTS.comparisonMode;
}

function readTemplateFile(configuration: Configuration): string | undefined {
	const value = configuration.get<string | undefined>(
		'templateFile',
		undefined,
	);
	return typeof value === 'string' && value.length > 0 ? value : undefined;
}

function isValidNotificationLevel(v: unknown): v is NotificationLevel {
	return v === 'all' || v === 'important' || v === 'silent';
}

function isValidComparisonMode(v: unknown): v is ComparisonMode {
	return v === 'auto' || v === 'manual' || v === 'template';
}
