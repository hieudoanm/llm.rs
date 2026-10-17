/**
 * The central error catalogue (FEATURE-18).
 *
 * Every backend error code maps to a user-facing description: a short title,
 * a plain-language message, an optional primary action, and whether a plain
 * retry can help. The catalogue is the single source of truth for the
 * guidance shown next to an error.
 */

import { API_BASE, type ApiError } from './api';

export interface ErrorGuidance {
  /** A short heading. */
  title: string;
  /** One or two sentences in plain language. */
  message: string;
  /** The label of the primary action. */
  action?: string;
  /** Where the primary action navigates. */
  actionHref?: string;
  /** May the user simply try again? */
  retryable: boolean;
  /** Extra bullet points shown in a disclosure. */
  details?: string[];
}

/** The placeholders a message may interpolate. */
export type GuidanceContext = Record<string, string | number | undefined>;

const format = (template: string, context: GuidanceContext): string =>
  template.replace(/\{(\w+)\}/g, (match, key: string) => {
    const value = context[key];
    return value === undefined || value === null ? match : String(value);
  });

/**
 * The catalogue. The message templates may hold `{placeholder}` markers that
 * are interpolated from the error details or from the client configuration.
 */
export const ERROR_GUIDANCE: Record<string, ErrorGuidance> = {
  VALIDATION_ERROR: {
    title: 'Check the form',
    message: 'Some values are not valid. The highlighted fields list what to correct.',
    retryable: false,
  },
  INVALID_COLUMN_MAP: {
    title: 'Fix the column map',
    message: 'The label column is also a feature, or a column in the map is not in the dataset.',
    action: 'Back to mapping',
    actionHref: '/jobs/new',
    retryable: false,
  },
  INVALID_LABEL_TYPE: {
    title: 'Wrong label type',
    message: 'The label column must be numeric for regression and categorical for classification.',
    action: 'Back to mapping',
    actionHref: '/jobs/new',
    retryable: false,
  },
  INSUFFICIENT_LABELS: {
    title: 'Not enough labels',
    message:
      'Semi-supervised needs at least 10 labelled rows, at least 2 classes, and at least 2 rows per class.',
    action: 'Back to mapping',
    actionHref: '/jobs/new',
    retryable: false,
  },
  INVALID_HYPERPARAMETERS: {
    title: 'Invalid setting',
    message: 'The hyperparameter value is outside the range this algorithm accepts.',
    action: 'Back to model',
    actionHref: '/jobs/new',
    retryable: false,
  },
  UNKNOWN_ALGORITHM: {
    title: 'Algorithm unavailable',
    message: 'The algorithm is not in the catalogue any more. Pick another one.',
    action: 'Back to model',
    actionHref: '/jobs/new',
    retryable: false,
  },
  TASK_ALGORITHM_MISMATCH: {
    title: 'Wrong algorithm',
    message: 'That algorithm does not support the selected task.',
    action: 'Back to model',
    actionHref: '/jobs/new',
    retryable: false,
  },
  DATASET_NOT_FOUND: {
    title: 'Dataset not found',
    message: 'The dataset was deleted. Upload the CSV again.',
    action: 'Upload again',
    actionHref: '/jobs/new',
    retryable: false,
  },
  JOB_NOT_FOUND: {
    title: 'Job not found',
    message: 'The job was deleted.',
    action: 'Back to jobs',
    actionHref: '/jobs',
    retryable: false,
  },
  ARTEFACT_NOT_FOUND: {
    title: 'File not found',
    message: 'The artefact is no longer on disk. Re-run the job.',
    action: 'Back to jobs',
    actionHref: '/jobs',
    retryable: false,
  },
  UPLOAD_TOO_LARGE: {
    title: 'File too large',
    message: 'The file exceeds the {limit} MiB upload limit. Split it or sample it first.',
    action: 'Choose another',
    retryable: false,
  },
  UNSUPPORTED_FILE_TYPE: {
    title: 'Not a CSV file',
    message: 'The upload must be a plain-text CSV with a header row.',
    action: 'Choose another',
    retryable: false,
  },
  MALFORMED_CSV: {
    title: 'Cannot read the CSV',
    message: 'The file could not be parsed as CSV. Check the delimiter and the header row.',
    action: 'Choose another',
    retryable: false,
  },
  IDEMPOTENCY_KEY_EXISTS: {
    title: 'Request already sent',
    message: 'A different payload was already submitted with the same idempotency key.',
    retryable: false,
  },
  TRAINING_FAILED: {
    title: 'Training failed',
    message: 'The model could not be fitted with this mapping and these settings.',
    action: 'Review settings',
    actionHref: '/jobs/new',
    retryable: false,
  },
  EXPORT_FAILED: {
    title: 'Export failed',
    message: 'The model trained, but the ONNX export failed.',
    action: 'Re-run job',
    retryable: true,
  },
  JOB_TIMEOUT: {
    title: 'Training timed out',
    message:
      'Training exceeded the {limit} s limit and was stopped. Reduce the rows or features, or raise the limit.',
    action: 'Review settings',
    actionHref: '/jobs/new',
    retryable: false,
  },
  JOB_RETRY_EXHAUSTED: {
    title: 'Temporary failure',
    message: 'The job failed {attempts} times because of a temporary environment error.',
    action: 'Retry',
    retryable: true,
  },
  WORKER_INTERRUPTED: {
    title: 'Job interrupted',
    message: 'The service restarted while the job was running. Start the job again.',
    action: 'Retry',
    retryable: true,
  },
  INTERNAL_ERROR: {
    title: 'Server error',
    message:
      'Something failed on the server. The request id is shown below when reporting the problem.',
    action: 'Retry',
    retryable: true,
  },
  SERVICE_UNAVAILABLE: {
    title: 'Service unavailable',
    message: 'The database is not reachable. Start the stack with `docker compose up -d`.',
    action: 'Retry',
    retryable: true,
  },
  NETWORK_ERROR: {
    title: 'Cannot reach the API',
    message: 'The request did not reach the backend at {apiUrl}. Check that the stack is running.',
    action: 'Retry',
    retryable: true,
  },
  REQUEST_TIMEOUT: {
    title: 'Request timed out',
    message: 'The server did not answer within {timeout} ms. It may be busy — try again.',
    action: 'Retry',
    retryable: true,
  },
  REQUEST_ABORTED: {
    title: 'Request cancelled',
    message: 'The request was cancelled before the server answered.',
    retryable: false,
  },
  INVALID_RESPONSE: {
    title: 'Unexpected response',
    message: 'The server answered with a response the app could not read.',
    action: 'Retry',
    retryable: true,
  },
  UNKNOWN_ERROR: {
    title: 'Server error',
    message:
      'Something failed on the server. The request id is shown below when reporting the problem.',
    action: 'Retry',
    retryable: true,
  },
};

/** The guidance used for a code that is not in the catalogue. */
export const FALLBACK_GUIDANCE: ErrorGuidance = {
  title: 'Server error',
  message:
    'Something failed on the server. The request id is shown below when reporting the problem.',
  action: 'Retry',
  retryable: true,
};

/** The default context used to interpolate the catalogue placeholders. */
export const defaultGuidanceContext = (): GuidanceContext => ({
  apiUrl: API_BASE,
  limit: 10,
  attempts: 2,
  timeout: 15000,
});

/**
 * Look up the guidance for an error.
 *
 * The backend's own code wins over the transport code, so a `VALIDATION_ERROR`
 * from the server is never flattened into the generic `HTTP_ERROR` guidance.
 */
export const guidanceFor = (
  error: Pick<ApiError, 'code' | 'serverCode' | 'details'> | null | undefined,
  context: GuidanceContext = {},
): ErrorGuidance => {
  if (!error) return FALLBACK_GUIDANCE;

  const key = error.serverCode ?? error.code;
  const entry = ERROR_GUIDANCE[key];
  if (!entry) return { ...FALLBACK_GUIDANCE };

  const details = (error.details ?? {}) as Record<string, unknown>;
  const merged: GuidanceContext = {
    ...defaultGuidanceContext(),
    ...Object.fromEntries(
      Object.entries(details).map(([name, value]) => [
        name,
        typeof value === 'string' || typeof value === 'number' ? value : undefined,
      ]),
    ),
    ...context,
  };

  return {
    ...entry,
    title: format(entry.title, merged),
    message: format(entry.message, merged),
    details: entry.details?.map((line) => format(line, merged)),
  };
};

/**
 * The extra bullets shown next to an error: the raw code and the fields the
 * server complained about.
 */
export const errorDetails = (
  error: Pick<ApiError, 'code' | 'serverCode' | 'details' | 'status'> | null | undefined,
): string[] => {
  if (!error) return [];
  const lines: string[] = [];
  const serverCode = error.serverCode;
  if (serverCode && serverCode !== error.code) lines.push(`Backend code: ${serverCode}`);
  if (error.status) lines.push(`HTTP status: ${error.status}`);
  const details = error.details ?? {};
  const fields = details.fields;
  if (Array.isArray(fields)) {
    for (const field of fields) {
      if (field && typeof field === 'object' && 'loc' in field) {
        const loc = (field as { loc?: unknown[] }).loc;
        const name = Array.isArray(loc) ? loc.join('.') : String(loc);
        const message = (field as { msg?: string }).msg;
        lines.push(message ? `${name}: ${message}` : name);
      }
    }
  }
  return lines;
};
