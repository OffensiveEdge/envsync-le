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
		safetyEnabled: readSafetyEnabled(configuration),
		fileSizeWarnBytes: readFileSizeWarnBytes(configuration),
		maxFilesWarn: readMaxFilesWarn(configuration),
		maxTotalSizeWarn: readMaxTotalSizeWarn(configuration),
		maxProcessingTimeWarn: readMaxProcessingTimeWarn(configuration),
		performanceEnabled: readPerformanceEnabled(configuration),
		performanceMaxDuration: readPerformanceMaxDuration(configuration),
		performanceMaxMemoryUsage: readPerformanceMaxMemoryUsage(configuration),
		performanceMaxCpuUsage: readPerformanceMaxCpuUsage(configuration),
		performanceMinThroughput: readPerformanceMinThroughput(configuration),
		performanceMaxCacheSize: readPerformanceMaxCacheSize(configuration),
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

function readSafetyEnabled(configuration: Configuration): boolean {
	return Boolean(configuration.get('safety.enabled', false));
}

function readFileSizeWarnBytes(configuration: Configuration): number {
	const MIN_SIZE = 1024;
	const DEFAULT_SIZE = 1024 * 1024;
	const value = Number(
		configuration.get('safety.fileSizeWarnBytes', DEFAULT_SIZE),
	);
	return Math.max(MIN_SIZE, value);
}

function readMaxFilesWarn(configuration: Configuration): number {
	const MIN_FILES = 1;
	const value = Number(configuration.get('safety.maxFilesWarn', 50));
	return Math.max(MIN_FILES, value);
}

function readMaxTotalSizeWarn(configuration: Configuration): number {
	const MIN_SIZE = 1024 * 1024;
	const DEFAULT_SIZE = 5 * 1024 * 1024;
	const value = Number(
		configuration.get('safety.maxTotalSizeWarn', DEFAULT_SIZE),
	);
	return Math.max(MIN_SIZE, value);
}

function readMaxProcessingTimeWarn(configuration: Configuration): number {
	const MIN_TIME = 1000;
	const value = Number(configuration.get('safety.maxProcessingTimeWarn', 5000));
	return Math.max(MIN_TIME, value);
}

function readPerformanceEnabled(configuration: Configuration): boolean {
	return Boolean(configuration.get('performance.enabled', true));
}

function readPerformanceMaxDuration(configuration: Configuration): number {
	const MIN_DURATION = 1000;
	const value = Number(configuration.get('performance.maxDuration', 5000));
	return Math.max(MIN_DURATION, value);
}

function readPerformanceMaxMemoryUsage(configuration: Configuration): number {
	const MIN_MEMORY = 1048576;
	const DEFAULT_MEMORY = 104857600;
	const value = Number(
		configuration.get('performance.maxMemoryUsage', DEFAULT_MEMORY),
	);
	return Math.max(MIN_MEMORY, value);
}

function readPerformanceMaxCpuUsage(configuration: Configuration): number {
	const MIN_CPU = 100000;
	const DEFAULT_CPU = 1000000;
	const value = Number(
		configuration.get('performance.maxCpuUsage', DEFAULT_CPU),
	);
	return Math.max(MIN_CPU, value);
}

function readPerformanceMinThroughput(configuration: Configuration): number {
	const MIN_THROUGHPUT = 100;
	const value = Number(configuration.get('performance.minThroughput', 1000));
	return Math.max(MIN_THROUGHPUT, value);
}

function readPerformanceMaxCacheSize(configuration: Configuration): number {
	const MIN_CACHE = 100;
	const value = Number(configuration.get('performance.maxCacheSize', 1000));
	return Math.max(MIN_CACHE, value);
}

function isValidNotificationLevel(v: unknown): v is NotificationLevel {
	return typeof v === 'string' && ['all', 'important', 'silent'].includes(v);
}

function isValidComparisonMode(v: unknown): v is ComparisonMode {
	return typeof v === 'string' && ['auto', 'manual', 'template'].includes(v);
}
