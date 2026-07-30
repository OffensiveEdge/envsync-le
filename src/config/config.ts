import type { Configuration } from '../interfaces';
import type {
	ComparisonMode,
	DotSyncConfig,
	NotificationLevel,
} from '../types';

export function readConfig(configuration: Configuration): DotSyncConfig {
	return Object.freeze({
		enabled: readEnabledSetting(configuration),
		watchPatterns: readWatchPatterns(configuration),
		excludePatterns: readExcludePatterns(configuration),
		notificationLevel: readNotificationLevel(configuration),
		statusBarEnabled: readStatusBarEnabled(configuration),
		debounceMs: readDebounceMs(configuration),
		ignoreComments: readIgnoreComments(configuration),
		caseSensitive: readCaseSensitive(configuration),
		telemetryEnabled: readTelemetryEnabled(configuration),
		comparisonMode: readComparisonMode(configuration),
		compareOnlyFiles: readCompareOnlyFiles(configuration),
		templateFile: readTemplateFile(configuration),
		temporaryIgnore: readTemporaryIgnore(configuration),
	});
}

function readEnabledSetting(configuration: Configuration): boolean {
	return Boolean(configuration.get('enabled', true));
}

function readWatchPatterns(configuration: Configuration): readonly string[] {
	const patterns = configuration.get('watchPatterns', ['.env*']) as string[];
	return Object.freeze([...patterns]);
}

function readExcludePatterns(configuration: Configuration): readonly string[] {
	const patterns = configuration.get('excludePatterns', [
		'.env.*.local',
	]) as string[];
	return Object.freeze([...patterns]);
}

function readNotificationLevel(
	configuration: Configuration,
): NotificationLevel {
	const rawValue = configuration.get(
		'notificationLevel',
		configuration.get('notificationsLevel', 'important'),
	) as unknown as string;

	if (isValidNotificationLevel(rawValue)) {
		return rawValue;
	}

	return 'important';
}

function readStatusBarEnabled(configuration: Configuration): boolean {
	return Boolean(configuration.get('statusBar.enabled', true));
}

function readDebounceMs(configuration: Configuration): number {
	const MIN_DEBOUNCE = 100;
	const value = Number(configuration.get('debounceMs', 1000));
	return Math.max(MIN_DEBOUNCE, value);
}

function readIgnoreComments(configuration: Configuration): boolean {
	return Boolean(configuration.get('ignoreComments', true));
}

function readCaseSensitive(configuration: Configuration): boolean {
	return Boolean(configuration.get('caseSensitive', true));
}

function readTelemetryEnabled(configuration: Configuration): boolean {
	return Boolean(configuration.get('telemetryEnabled', false));
}

function readComparisonMode(configuration: Configuration): ComparisonMode {
	const rawValue = configuration.get('comparisonMode', 'auto');

	if (isValidComparisonMode(rawValue)) {
		return rawValue;
	}

	return 'auto';
}

function readCompareOnlyFiles(configuration: Configuration): readonly string[] {
	const files = configuration.get('compareOnlyFiles', []) as string[];
	return Object.freeze([...files]);
}

function readTemplateFile(configuration: Configuration): string | undefined {
	return configuration.get(
		'templateFile',
		undefined as unknown as string | undefined,
	);
}

function readTemporaryIgnore(configuration: Configuration): readonly string[] {
	const files = configuration.get('temporaryIgnore', []) as string[];
	return Object.freeze([...files]);
}

function isValidNotificationLevel(v: unknown): v is NotificationLevel {
	return typeof v === 'string' && ['all', 'important', 'silent'].includes(v);
}

function isValidComparisonMode(v: unknown): v is ComparisonMode {
	return typeof v === 'string' && ['auto', 'manual', 'template'].includes(v);
}
