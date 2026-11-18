import createClient from 'openapi-fetch';
import type { paths } from './schema.d.ts';

/**
 * The one API client instance; only `src/lib/api` may import it (enforced by ESLint).
 * It needs no base URL, because the Rust binary serves the app and `/api` from one origin.
 */
export const client = createClient<paths>();
