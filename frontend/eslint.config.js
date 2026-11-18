import prettier from 'eslint-config-prettier';
import path from 'node:path';
import js from '@eslint/js';
import svelte from 'eslint-plugin-svelte';
import { defineConfig, includeIgnoreFile } from 'eslint/config';
import globals from 'globals';
import ts from 'typescript-eslint';

const API_CLIENT_RULE =
	'Only src/lib/api may hold the API client. Import an operation function from #lib/api instead.';

const gitignorePath = path.resolve(import.meta.dirname, '.gitignore');

export default defineConfig(
	includeIgnoreFile(gitignorePath),
	js.configs.recommended,
	ts.configs.recommended,
	svelte.configs.recommended,
	prettier,
	svelte.configs.prettier,
	{
		languageOptions: { globals: { ...globals.browser, ...globals.node } },
		rules: {
			// typescript-eslint strongly recommend that you do not use the no-undef lint rule on TypeScript projects.
			// see: https://typescript-eslint.io/troubleshooting/faqs/eslint/#i-get-errors-from-the-no-undef-rule-about-global-variables-not-being-defined-even-though-there-are-no-typescript-errors
			'no-undef': 'off'
		}
	},
	{
		files: ['**/*.svelte', '**/*.svelte.ts', '**/*.svelte.js'],
		languageOptions: {
			parserOptions: {
				projectService: true,
				extraFileExtensions: ['.svelte'],
				parser: ts.parser
			}
		}
	},
	{
		// The frontend never renders raw HTML (spec rule 6.5.4).
		rules: { 'svelte/no-at-html-tags': 'error' }
	},
	{
		// Only `src/lib/api` holds the API client instance (spec rule 2.3.2).
		// Everything else calls the operation functions that folder exports.
		files: ['src/**'],
		ignores: ['src/lib/api/**'],
		rules: {
			'no-restricted-imports': [
				'error',
				{
					paths: [{ name: 'openapi-fetch', message: API_CLIENT_RULE }],
					patterns: [{ group: ['**/api/client.ts'], message: API_CLIENT_RULE }]
				}
			]
		}
	},
	// Generated from openapi.json; never edited or formatted by hand.
	{ ignores: ['src/lib/api/schema.d.ts'] }
);
