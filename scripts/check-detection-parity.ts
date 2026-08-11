/**
 * Fails when the extension's dotenv detection drifts from the shared
 * corpus, which the Rust CLI (crate/) also builds against.
 *
 * - detection.json `parsing`: the keys each corpus file yields and the
 *   parse errors it reports, including the awkward ones — a multi-line
 *   quoted value, and an unterminated quote swallowing the rest of the
 *   file.
 * - detection.json `classification`: which names are dotenv files and
 *   what type each is. Segment-based, not substring-based: `.env.production`
 *   is production and `app.device.env` is not development.
 * - detection.json `comparison`: the report, in both modes and both
 *   casings. `lastChecked` is excluded — see crate/SPEC.md, "The report
 *   has no timestamp".
 * - mcp-compare-env-files.json: the `compare_env_files` tool, which BOTH
 *   MCP servers offer and must answer identically.
 *
 * This checks only the extension's side. `cargo test` runs the crate's
 * implementation over the same files.
 *
 * Run: bun scripts/check-detection-parity.ts
 */
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { compareFiles } from '../src/detection/comparator';
import { detectFileType, isEnvFileName } from '../src/detection/heuristics';
import { parseDotenvFile } from '../src/detection/parser';
import { TOOLS } from '../src/mcp/tools';
import type { DotenvFile } from '../src/types';

const ROOT = join(import.meta.dir, '..');
/** The corpus lives inside the crate so the published package is self-contained. */
const CORPUS = join(ROOT, 'crate', 'fixtures');
const failures: string[] = [];

function fail(message: string): void {
	failures.push(message);
}

function deepEqual(a: unknown, b: unknown): boolean {
	if (a === b) return true;
	if (Array.isArray(a) && Array.isArray(b)) {
		if (a.length !== b.length) return false;
		return a.every((item, i) => deepEqual(item, b[i]));
	}
	if (typeof a !== 'object' || typeof b !== 'object') return false;
	if (a === null || b === null) return false;
	const keysA = Object.keys(a).sort();
	const keysB = Object.keys(b).sort();
	if (!deepEqual(keysA, keysB)) return false;
	return keysA.every((key) =>
		deepEqual(
			(a as Record<string, unknown>)[key],
			(b as Record<string, unknown>)[key],
		),
	);
}

function readCorpus(name: string): unknown {
	return JSON.parse(readFileSync(join(CORPUS, name), 'utf8'));
}

/**
 * A corpus document, by its **logical** name.
 *
 * The names are `.env`, `.env.production` and so on, because
 * classification is one of the things being tested and it reads the
 * name. The files on disk drop the leading dot: `cargo package` skips
 * dotfiles, so a corpus of them would ship a crate that cannot run its
 * own tests. The mapping lives here and in `crate/src/detect/corpus.rs`.
 */
function readDocument(name: string): string {
	return readFileSync(join(CORPUS, 'documents', `dot${name}`), 'utf8');
}

interface Corpus {
	readonly parsing: ReadonlyArray<{
		file: string;
		keys: readonly string[];
		errors: ReadonlyArray<{ type: string; message: string }>;
	}>;
	readonly classification: ReadonlyArray<{
		path: string;
		isEnv: boolean;
		type: string;
	}>;
	readonly comparison: ReadonlyArray<{
		name: string;
		files: readonly string[];
		options: Record<string, unknown>;
		expected: unknown;
	}>;
}

function asFile(name: string): DotenvFile {
	return {
		path: name,
		type: detectFileType(name),
		keys: [...parseDotenvFile(readDocument(name), name).keys],
		lastModified: 0,
	};
}

function checkParsing(corpus: Corpus): void {
	if (corpus.parsing.length === 0) fail('the corpus parses no documents');
	for (const testCase of corpus.parsing) {
		const result = parseDotenvFile(readDocument(testCase.file), testCase.file);
		const actual = {
			keys: [...result.keys],
			errors: result.errors.map((error) => ({
				type: error.type,
				message: error.message,
			})),
		};
		if (!deepEqual(actual, { keys: testCase.keys, errors: testCase.errors })) {
			fail(
				`parsing "${testCase.file}":\n  expected: ${JSON.stringify({ keys: testCase.keys, errors: testCase.errors })}\n  got:      ${JSON.stringify(actual)}`,
			);
		}
	}
}

function checkClassification(corpus: Corpus): void {
	if (corpus.classification.length === 0) fail('the corpus classifies no names');
	for (const testCase of corpus.classification) {
		const actual = {
			path: testCase.path,
			isEnv: isEnvFileName(testCase.path),
			type: detectFileType(testCase.path),
		};
		if (!deepEqual(actual, testCase)) {
			fail(
				`classification "${testCase.path}": expected ${JSON.stringify(testCase)}, got ${JSON.stringify(actual)}`,
			);
		}
	}
}

function checkComparison(corpus: Corpus): void {
	if (corpus.comparison.length === 0) fail('the corpus compares nothing');
	for (const testCase of corpus.comparison) {
		// biome-ignore lint/suspicious/noExplicitAny: the corpus stores options as plain JSON
		const report = compareFiles(testCase.files.map(asFile), testCase.options as any);
		const { lastChecked, ...rest } = report;
		const actual = {
			...rest,
			files: rest.files.map((file) => ({
				path: file.path,
				type: file.type,
				keys: file.keys.length,
			})),
		};
		if (!deepEqual(actual, testCase.expected)) {
			fail(
				`comparison "${testCase.name}":\n  expected: ${JSON.stringify(testCase.expected)}\n  got:      ${JSON.stringify(actual)}`,
			);
		}
	}
}

/**
 * `compare_env_files` is offered by BOTH MCP servers. They are meant to
 * be the same tool, not two similar ones, so the same corpus runs
 * against both: this function here, and `crate/src/mcp/compare.rs`'s own
 * test there.
 */
async function checkMcpCompare(): Promise<void> {
	const cases = readCorpus('mcp-compare-env-files.json') as ReadonlyArray<{
		name: string;
		files?: readonly string[];
		arguments: Record<string, unknown>;
		expected?: unknown;
		expectedError?: string;
	}>;

	const tool = TOOLS.find((t) => t.name === 'compare_env_files');
	if (!tool) {
		fail('the extension no longer offers compare_env_files');
		return;
	}

	for (const testCase of cases) {
		const args: Record<string, unknown> = { ...testCase.arguments };
		if (testCase.files !== undefined) {
			args.files = testCase.files.map((name) => ({
				path: name,
				content: readDocument(name),
			}));
		}

		if (testCase.expectedError !== undefined) {
			try {
				await tool.handler(args);
				fail(
					`mcp compare "${testCase.name}": expected it to fail with ${JSON.stringify(testCase.expectedError)}`,
				);
			} catch (error) {
				const message = error instanceof Error ? error.message : String(error);
				if (message !== testCase.expectedError) {
					fail(
						`mcp compare "${testCase.name}": expected error ${JSON.stringify(testCase.expectedError)}, got ${JSON.stringify(message)}`,
					);
				}
			}
			continue;
		}

		const actual = await tool.handler(args);
		if (!deepEqual(actual, testCase.expected)) {
			fail(
				`mcp compare "${testCase.name}":\n  expected: ${JSON.stringify(testCase.expected)}\n  got:      ${JSON.stringify(actual)}`,
			);
		}
	}
}

/**
 * A dotenv file is where credentials live, so the answer must never
 * carry one. Checked against the corpus rather than trusted, because a
 * value leaking into a report is the one failure this tool cannot have.
 */
function checkNoValuesLeak(): void {
	const secrets = ['postgres://', 'redis://', 'sentry.io', 'hunter2', '-----BEGIN-----'];
	for (const name of ['detection.json', 'mcp-compare-env-files.json']) {
		const text = readFileSync(join(CORPUS, name), 'utf8');
		for (const secret of secrets) {
			if (text.includes(secret)) {
				fail(`${name} carries a value (${secret}) — only key names may appear`);
			}
		}
	}
}

const corpus = readCorpus('detection.json') as Corpus;
checkParsing(corpus);
checkClassification(corpus);
checkComparison(corpus);
checkNoValuesLeak();
await checkMcpCompare();

if (failures.length > 0) {
	console.error(`Detection parity FAILED (${failures.length}):\n`);
	for (const failure of failures) {
		console.error(`- ${failure}\n`);
	}
	process.exit(1);
}
console.log(
	'OK: every corpus case reproduces under the extension, nothing leaks a value, and both MCP servers agree.',
);
