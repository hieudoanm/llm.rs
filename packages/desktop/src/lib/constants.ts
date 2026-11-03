/**
 * The client-side limits of the API contract.
 *
 * They mirror the backend rules so the UI can validate before it sends a
 * request; the backend stays the authority.
 */

/** The maximum length of a job name. */
export const MAX_NAME_LENGTH = 120;

/** The maximum number of tags a job may carry. */
export const MAX_TAGS = 10;

/** The maximum length of a single tag. */
export const MAX_TAG_LENGTH = 32;

/** The maximum number of jobs that may be compared at once. */
export const MAX_COMPARE_JOBS = 5;

/** The maximum number of jobs a bulk archive may hold. */
export const MAX_BUNDLE_JOBS = 10;

/** The upload limit, in mebibytes. */
export const MAX_UPLOAD_MB = 10;

/** The upload limit, in bytes. */
export const MAX_UPLOAD_BYTES = MAX_UPLOAD_MB * 1024 * 1024;

/** The statuses a job never leaves. */
export const TERMINAL_STATUSES = ['completed', 'failed'] as const;

/** The lifecycle order of a job status. */
export const JOB_STATUS_ORDER = [
  'queued',
  'validating',
  'training',
  'evaluating',
  'exporting',
  'retrying',
  'completed',
  'failed',
] as const;
