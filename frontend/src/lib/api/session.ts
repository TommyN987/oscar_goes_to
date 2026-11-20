import { client } from './client.ts';
import type { components } from './schema.d.ts';

export type WhoAmI = components['schemas']['WhoAmI'];

/**
 * Reads the signed-in player through `fetch`, which a load function passes in.
 * `null` means the request carries no live session.
 */
export async function readWhoAmI(fetch: typeof globalThis.fetch): Promise<WhoAmI | null> {
	const { data, response } = await client.GET('/api/session', { fetch });
	if (response.status === 401) {
		return null;
	}
	if (!data) {
		throw new Error(`Who am I answered ${response.status}.`);
	}
	return data;
}

/** Signs the player out on every device. */
export async function signOut(fetch: typeof globalThis.fetch): Promise<void> {
	const { response } = await client.POST('/api/session/sign-out', { fetch });
	if (!response.ok) {
		throw new Error(`Sign out answered ${response.status}.`);
	}
}
