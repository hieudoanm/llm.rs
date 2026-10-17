import type { components } from '@/lib/apiTypes';

/** One downloadable file of a job, straight from the contract. */
export type Artefact = components['schemas']['ArtefactResponse'];

/**
 * The full job detail (FEATURE-13/18).
 *
 * Taken from the generated contract rather than retyped, so a schema change in
 * the OpenAPI document surfaces here at build time.
 */
export type Job = components['schemas']['JobResponse'];
