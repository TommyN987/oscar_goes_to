import { page } from 'vitest/browser';
import { expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import ApiStatus from './ApiStatus.svelte';

it('shows the status it is given', async () => {
	render(ApiStatus, { status: 'ok' });

	await expect.element(page.getByText('API status: ok')).toBeVisible();
});
