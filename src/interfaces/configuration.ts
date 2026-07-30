export interface Configuration {
	/**
	 * Get a configuration value with optional default
	 */
	get<T>(key: string, defaultValue: T): T;

	/**
	 * Check if a configuration key exists
	 */
	has(key: string): boolean;
}
