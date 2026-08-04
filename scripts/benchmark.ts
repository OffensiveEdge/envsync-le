/**
 * Measure real throughput. Run with `bun run benchmark`.
 *
 * EnvSync-LE compares key sets across .env files, so the work scales with
 * files x keys rather than bytes of a single document. Cases below vary both.
 *
 * Numbers are machine-specific and never asserted in CI.
 */
import { cpus, totalmem } from 'node:os';
import { compareFiles } from '../src/detection/comparator';
import type { DotenvFile } from '../src/types';

interface Case {
	readonly label: string;
	readonly arg: string;
	readonly build: () => string;
	readonly files: () => readonly DotenvFile[];
}

function makeFiles(count: number, keys: number, drift: number): readonly DotenvFile[] {
	return Array.from({ length: count }, (_, f) => ({
		path: `/repo/.env.${f}`,
		type: 'env' as never,
		keys: Array.from({ length: keys }, (_, k) => `KEY_${k}`).filter(
			(_, k) => (f === 0 ? true : k % drift !== 0),
		),
		lastModified: 0,
	}));
}

const SETS = {
	small: makeFiles(4, 500, 17),
	wide: makeFiles(24, 500, 13),
	deep: makeFiles(4, 8000, 29),
};

const CASES: readonly Case[] = [
	{ label: '4 files x 500 keys', arg: 'small', build: () => JSON.stringify(SETS.small), files: () => SETS.small },
	{ label: '24 files x 500 keys', arg: 'wide', build: () => JSON.stringify(SETS.wide), files: () => SETS.wide },
	{ label: '4 files x 8,000 keys', arg: 'deep', build: () => JSON.stringify(SETS.deep), files: () => SETS.deep },
];

async function run(_content: string, c: Case): Promise<number> {
	const report = compareFiles(c.files());
	return c.files().reduce((n, f) => n + f.keys.length, 0);
}

const WARMUP = 2;
const RUNS = 7;

function median(xs: readonly number[]): number {
	const s = [...xs].sort((a, b) => a - b);
	const mid = Math.floor(s.length / 2);
	return s.length % 2 ? (s[mid] as number) : ((s[mid - 1] as number) + (s[mid] as number)) / 2;
}

async function main(): Promise<void> {
	const results: Array<Record<string, unknown>> = [];
	for (const c of CASES) {
		const content = c.build();
		const bytes = Buffer.byteLength(content, 'utf8');
		for (let i = 0; i < WARMUP; i++) await run(content, c);
		const durations: number[] = [];
		let count = 0;
		for (let i = 0; i < RUNS; i++) {
			const t0 = performance.now();
			count = await run(content, c);
			durations.push(performance.now() - t0);
		}
		const ms = median(durations);
		results.push({
			label: c.label,
			bytes,
			lines: content.split('\n').length,
			extracted: count,
			ms: Number(ms.toFixed(2)),
			perSecond: count > 0 ? Math.round(count / (ms / 1000)) : null,
			mbPerSecond: Number((bytes / 1_048_576 / (ms / 1000)).toFixed(1)),
		});
		console.log(`${c.label.padEnd(22)} ${(bytes / 1_048_576).toFixed(2)} MB  ${String(count).padStart(7)}  ${ms.toFixed(2)} ms`);
	}
	const cpu = cpus()[0]?.model ?? 'unknown CPU';
	await Bun.write('benchmark-results.json', `${JSON.stringify({ host: `${cpu}, ${Math.round(totalmem() / 1_073_741_824)} GB RAM, Node ${process.versions.node}`, runs: RUNS, results }, null, 2)}\n`);
	console.log('\nwrote benchmark-results.json');
}

await main();
