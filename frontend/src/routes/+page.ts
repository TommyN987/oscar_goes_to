import { readLiveness } from '#lib/api/health.ts';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch }) => ({
	liveness: await readLiveness(fetch)
});
