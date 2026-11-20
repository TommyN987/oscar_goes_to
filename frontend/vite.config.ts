import { defineConfig } from 'vitest/config';
import { playwright } from '@vitest/browser-playwright';
import tailwindcss from '@tailwindcss/vite';
import adapter from '@sveltejs/adapter-static';
import { sveltekit } from '@sveltejs/kit/vite';

// The port `just dev` starts the Rust backend on.
const LOCAL_BACKEND = 'http://localhost:8080';

export default defineConfig({
	plugins: [
		tailwindcss(),
		sveltekit({
			compilerOptions: {
				// Force runes mode for the project, except for libraries. Can be removed in svelte 6.
				runes: ({ filename }) =>
					filename.split(/[/\\]/).includes('node_modules') ? undefined : true
			},
			// A static SPA: the Rust binary serves this fallback page for every non-file path.
			adapter: adapter({ fallback: '200.html' }),
			// The content security policy (spec rule 6.5.1). SvelteKit writes it into the fallback page as a
			// <meta> tag and adds the hash of its own inline script, so no other inline script can run.
			// The server sends `frame-ancestors`, which a <meta> policy cannot carry
			// (crates/adapters/src/http/security_headers.rs).
			csp: {
				mode: 'hash',
				directives: {
					'default-src': ['self'],
					'script-src': ['self'],
					'style-src': ['self'],
					// Style attributes stay allowed: SvelteKit hides its route announcer with one,
					// and an attribute, unlike a <style> element, cannot hold a selector or load anything.
					'style-src-attr': ['unsafe-inline'],
					// Vite inlines small images, the favicon among them, as data URLs, and so does daisyUI.
					'img-src': ['self', 'data:'],
					'object-src': ['none'],
					'base-uri': ['self'],
					'form-action': ['self']
				}
			}
		})
	],
	server: {
		// Same origin in development as in production: `/api` reaches the local backend.
		proxy: { '/api': LOCAL_BACKEND }
	},
	test: {
		expect: { requireAssertions: true },
		projects: [
			{
				extends: './vite.config.ts',
				test: {
					name: 'component',
					browser: {
						enabled: true,
						provider: playwright(),
						instances: [{ browser: 'chromium', headless: true }]
					},
					include: ['src/**/*.svelte.{test,spec}.{js,ts}']
				}
			}
		]
	}
});
