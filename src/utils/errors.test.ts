import { describe, expect, it } from 'vitest';
import { errorMessage, sanitizeErrorMessage } from './errors';

describe('sanitizeErrorMessage', () => {
	it('redacts user directories', () => {
		expect(sanitizeErrorMessage('ENOENT /Users/jane/project/.env')).toBe(
			'ENOENT /Users/***/project/.env',
		);
		expect(sanitizeErrorMessage('read /home/jane/app/.env failed')).toBe(
			'read /home/***/app/.env failed',
		);
		expect(sanitizeErrorMessage('open C:\\Users\\jane\\app')).toBe(
			'open C:\\Users\\***\\app',
		);
	});

	it('redacts credential-shaped fragments', () => {
		expect(sanitizeErrorMessage('bad password=hunter2 in file')).toBe(
			'bad password=*** in file',
		);
		expect(sanitizeErrorMessage('token: abc123')).toBe('token=***');
		expect(sanitizeErrorMessage('api key=xyz')).toBe('api key=***');
	});
});

describe('errorMessage', () => {
	it('unwraps Error instances and stringifies the rest', () => {
		expect(errorMessage(new Error('boom'))).toBe('boom');
		expect(errorMessage('plain')).toBe('plain');
		expect(errorMessage(42)).toBe('42');
	});
});
