import { client } from './client.ts';
import type { components } from './schema.d.ts';

export type Liveness = components['schemas']['Liveness'];

/** Reads the backend's liveness through `fetch`, which a load function passes in. */
export async function readLiveness(fetch: typeof globalThis.fetch): Promise<Liveness> {
	const { data, response } = await client.GET('/api/health', { fetch });
	if (!data) {
		throw new Error(`The liveness check answered ${response.status}.`);
	}
	return data;
}
