import type { ParseError } from '../types';

/**
 * The one result shape every tool returns.
 *
 * The engines across the family disagree — some async, some sync, six different
 * result envelopes. Normalising here rather than in the engines keeps the
 * characterization goldens untouched and the extension's behaviour unchanged.
 */
export interface ToolEnvelope<T> {
	readonly ok: boolean;
	readonly data: T;
	readonly diagnostics: readonly Diagnostic[];
	readonly meta: EnvelopeMeta;
}

export interface Diagnostic {
	readonly severity: 'warning' | 'error';
	readonly code: string;
	readonly message: string;
}

export interface EnvelopeMeta {
	readonly tool: string;
	readonly count: number;
	readonly truncated: boolean;
}

/** Above this an unbounded extraction would flood an agent's context window. */
export const DEFAULT_MAX_RESULTS = 500;
export const MAX_MAX_RESULTS = 5000;

/**
 * `ok` reports whether the comparison ran, not whether the files agreed.
 *
 * Files that are out of sync are the *finding*, not a failure — that is the
 * whole question being asked. Only a file that could not be parsed makes the
 * answer untrustworthy, so only that sets `ok: false`.
 */
export function isOk(diagnostics: readonly Diagnostic[]): boolean {
	return !diagnostics.some((d) => d.severity === 'error');
}

export function toDiagnostics(
	errors: readonly ParseError[],
): readonly Diagnostic[] {
	return errors.map((e) => ({
		severity: 'error' as const,
		code: e.type,
		message: `${e.filepath}: ${e.message}`,
	}));
}

/** Apply the result cap, reporting honestly whether anything was dropped. */
export function capped<T>(
	items: readonly T[],
	maxResults: number,
): { readonly items: readonly T[]; readonly truncated: boolean } {
	if (items.length <= maxResults) {
		return { items, truncated: false };
	}
	return { items: items.slice(0, maxResults), truncated: true };
}

export function envelope<T>(
	tool: string,
	data: T,
	count: number,
	diagnostics: readonly Diagnostic[],
	truncated: boolean,
): ToolEnvelope<T> {
	return {
		ok: isOk(diagnostics),
		data,
		diagnostics,
		meta: { tool, count, truncated },
	};
}

/** Read a bounded integer argument, rejecting values a tool cannot honour. */
export function readMaxResults(args: Record<string, unknown>): number {
	const raw = args.maxResults;
	if (raw === undefined) return DEFAULT_MAX_RESULTS;
	if (typeof raw !== 'number' || !Number.isInteger(raw) || raw < 1) {
		throw new Error('maxResults must be a positive integer');
	}
	return Math.min(raw, MAX_MAX_RESULTS);
}

/** Read a required string argument with a message naming the argument. */
export function readString(
	args: Record<string, unknown>,
	name: string,
): string {
	const value = args[name];
	if (typeof value !== 'string') {
		throw new Error(`${name} is required and must be a string`);
	}
	return value;
}
