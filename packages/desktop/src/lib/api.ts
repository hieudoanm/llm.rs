/**
 * Resilient API client (FEATURE-18).
 *
 * Every call goes through this module. The wrapper adds a timeout, retries with
 * full-jitter backoff for safe requests, honours an external `AbortSignal`, and
 * normalises every failure into a single `ApiError` with a discriminated `code`.
 */

// An empty value means "use the default" — the compose file passes an empty
// string when the variable is unset.
const configuredApiUrl = process.env.NEXT_PUBLIC_API_URL?.trim();
export const API_BASE = (configuredApiUrl || 'http://127.0.0.1:8000').replace(/\/$/, '');

/** Default timeout for a regular request, in milliseconds. */
export const DEFAULT_TIMEOUT_MS = 15000;

/** Default timeout for an upload, in milliseconds. */
export const UPLOAD_TIMEOUT_MS = 60000;

/** Default number of extra attempts after the first one. */
export const DEFAULT_RETRIES = 2;

/** Base of the exponential backoff between attempts, in milliseconds. */
export const DEFAULT_RETRY_BASE_MS = 300;

/** Upper bound of a single backoff wait, in milliseconds. */
export const MAX_RETRY_DELAY_MS = 5000;

/**
 * The transport-level error codes raised by the wrapper.
 *
 * The server's own error codes survive in `ApiError.serverCode`, so a backend
 * error is never flattened into `HTTP_ERROR`.
 */
export type ApiErrorCode =
  'NETWORK_ERROR' | 'REQUEST_TIMEOUT' | 'REQUEST_ABORTED' | 'HTTP_ERROR' | 'INVALID_RESPONSE';

export class ApiError extends Error {
  constructor(
    public code: ApiErrorCode,
    public message: string,
    public status: number,
    public requestId?: string,
    public details?: Record<string, unknown>,
    /** The backend's own error code, when the response carried an envelope. */
    public serverCode?: string,
  ) {
    super(message);
    this.name = 'ApiError';
  }

  /** True when the failure may be worth another attempt. */
  get retryable(): boolean {
    if (this.code === 'NETWORK_ERROR' || this.code === 'REQUEST_TIMEOUT') return true;
    if (this.code !== 'HTTP_ERROR') return false;
    return RETRYABLE_STATUSES.has(this.status);
  }
}

/** The HTTP statuses that are worth another attempt. */
const RETRYABLE_STATUSES = new Set([429, 502, 503, 504]);

/** The request options understood by the wrapper. */
export interface ApiRequestOptions {
  /** Abort the request after this many milliseconds. */
  timeoutMs?: number;
  /** Extra attempts after the first one. */
  retries?: number;
  /** Base of the exponential backoff between attempts. */
  retryBaseMs?: number;
  /** An external signal that cancels the request and all its retries. */
  signal?: AbortSignal;
}

const parseResponse = async (res: Response): Promise<unknown> => {
  const contentType = res.headers.get('content-type') ?? '';
  return contentType.includes('application/json') ? res.json() : res.text();
};

const buildError = (res: Response, body: unknown, action: string, serverCode: string): ApiError => {
  const envelope = body as {
    error?: { code?: string; message?: string; requestId?: string; details?: unknown };
  };
  const err = envelope?.error ?? {};
  return new ApiError(
    'HTTP_ERROR',
    err.message ?? `${action} failed with status ${res.status}`,
    res.status,
    err.requestId,
    err.details as Record<string, unknown>,
    serverCode,
  );
};

/** True when the request may be retried automatically. */
const isRetryableMethod = (method: string, headers: Headers): boolean => {
  const normalized = method.toUpperCase();
  if (normalized === 'GET' || normalized === 'HEAD') return true;
  // A POST is only safe to replay when the backend can deduplicate it.
  if (normalized === 'POST') return headers.has('Idempotency-Key');
  return false;
};

/** True when the failure justifies another attempt. */
const isRetryableFailure = (error: unknown): boolean => {
  if (error instanceof ApiError) return error.retryable;
  return false;
};

/**
 * The wait before attempt `n` (1-based): a random duration in
 * `[0, retryBaseMs * 2^(n-1))`, capped at 5 s.
 */
export const retryDelay = (attempt: number, retryBaseMs: number): number => {
  const ceiling = Math.min(MAX_RETRY_DELAY_MS, retryBaseMs * 2 ** (attempt - 1));
  return Math.random() * ceiling;
};

const sleep = (ms: number, signal?: AbortSignal): Promise<void> =>
  new Promise((resolve, reject) => {
    if (signal?.aborted) {
      reject(new ApiError('REQUEST_ABORTED', 'The request was aborted', 0));
      return;
    }
    const timer = setTimeout(() => {
      signal?.removeEventListener('abort', onAbort);
      resolve();
    }, ms);
    const onAbort = () => {
      clearTimeout(timer);
      reject(new ApiError('REQUEST_ABORTED', 'The request was aborted', 0));
    };
    signal?.addEventListener('abort', onAbort, { once: true });
  });

/**
 * Combine the caller's signal with the per-attempt timeout signal.
 *
 * The returned controller is aborted when either signal fires.
 */
const linkSignals = (
  timeoutMs: number,
  external?: AbortSignal,
): { signal: AbortSignal; cleanup: () => void } => {
  const controller = new AbortController();
  const timeoutId = setTimeout(() => controller.abort('timeout'), timeoutMs);
  const onExternalAbort = () => controller.abort('external');
  external?.addEventListener('abort', onExternalAbort, { once: true });
  return {
    signal: controller.signal,
    cleanup: () => {
      clearTimeout(timeoutId);
      external?.removeEventListener('abort', onExternalAbort);
    },
  };
};

interface InternalRequest extends ApiRequestOptions {
  method: string;
  headers: Headers;
  body?: BodyInit | null;
  action: string;
}

/**
 * Perform a request with the timeout, the retry budget, and the error
 * taxonomy applied.
 */
const performRequest = async <T>(path: string, options: InternalRequest): Promise<T> => {
  const {
    method,
    headers,
    body,
    action,
    timeoutMs = DEFAULT_TIMEOUT_MS,
    retries = DEFAULT_RETRIES,
    retryBaseMs = DEFAULT_RETRY_BASE_MS,
    signal,
  } = options;

  const url = `${API_BASE}/${String(path).replace(/^\//, '')}`;
  const retryable = isRetryableMethod(method, headers);
  const maxAttempts = retryable ? Math.max(0, retries) + 1 : 1;

  let lastError: unknown;

  for (let attempt = 1; attempt <= maxAttempts; attempt += 1) {
    if (signal?.aborted) {
      throw new ApiError('REQUEST_ABORTED', 'The request was aborted', 0);
    }

    const { signal: attemptSignal, cleanup } = linkSignals(timeoutMs, signal);
    try {
      const res = await fetch(url, { method, headers, body, signal: attemptSignal });

      if (res.status === 204) return undefined as T;

      const contentType = res.headers.get('content-type') ?? '';
      if (!res.ok) {
        const errorBody = await parseResponse(res);
        const envelope = errorBody as { error?: { code?: string } } | null;
        const serverCode = envelope?.error?.code ?? 'UNKNOWN_ERROR';
        const error = buildError(res, errorBody, action, serverCode);
        if (attempt < maxAttempts && isRetryableFailure(error)) {
          lastError = error;
          await sleep(retryDelay(attempt, retryBaseMs), signal);
          continue;
        }
        throw error;
      }

      if (!contentType.includes('application/json')) {
        return (await res.text()) as T;
      }

      try {
        return (await res.json()) as T;
      } catch {
        throw new ApiError(
          'INVALID_RESPONSE',
          'The server returned a response that could not be parsed',
          res.status,
        );
      }
    } catch (cause) {
      if (cause instanceof ApiError) {
        if (cause.code === 'REQUEST_ABORTED') {
          // The caller cancelled: never retry.
          throw cause;
        }
        if (cause.code === 'HTTP_ERROR' || cause.code === 'INVALID_RESPONSE') {
          throw cause;
        }
      }

      const aborted = attemptSignal.aborted;
      const timedOut = aborted && attemptSignal.reason === 'timeout';
      const cancelled = aborted && !timedOut;

      const error = cancelled
        ? new ApiError('REQUEST_ABORTED', 'The request was aborted', 0)
        : timedOut
          ? new ApiError(
              'REQUEST_TIMEOUT',
              `The request did not answer within ${timeoutMs} ms`,
              0,
              undefined,
              { timeoutMs },
            )
          : new ApiError('NETWORK_ERROR', 'The request could not reach the API', 0);

      if (attempt < maxAttempts && isRetryableFailure(error)) {
        lastError = error;
        await sleep(retryDelay(attempt, retryBaseMs), signal);
        continue;
      }
      throw error;
    } finally {
      cleanup();
    }
  }

  throw lastError instanceof ApiError
    ? lastError
    : new ApiError('NETWORK_ERROR', 'The request could not reach the API', 0);
};

export const apiRequest = async <T = unknown>(
  input: RequestInfo | URL,
  init?: RequestInit & ApiRequestOptions,
): Promise<T> => {
  const { timeoutMs, retries, retryBaseMs, signal, ...rest } = init ?? {};
  const headers = new Headers(rest.headers);
  if (!headers.has('Content-Type')) headers.set('Content-Type', 'application/json');

  return performRequest<T>(String(input), {
    method: rest.method ?? 'GET',
    headers,
    body: rest.body,
    action: 'Request',
    timeoutMs,
    retries,
    retryBaseMs,
    signal,
  });
};

export const apiPost = <T = unknown>(
  path: string,
  body?: unknown,
  init?: RequestInit & ApiRequestOptions,
): Promise<T> =>
  apiRequest<T>(path, {
    method: 'POST',
    body: body ? JSON.stringify(body) : undefined,
    ...init,
  });

export const apiGet = <T = unknown>(
  path: string,
  init?: RequestInit & ApiRequestOptions,
): Promise<T> => apiRequest<T>(path, { method: 'GET', ...init });

export const apiDelete = <T = unknown>(
  path: string,
  init?: RequestInit & ApiRequestOptions,
): Promise<T> => apiRequest<T>(path, { method: 'DELETE', ...init });

export const apiPatch = <T = unknown>(
  path: string,
  body?: unknown,
  init?: RequestInit & ApiRequestOptions,
): Promise<T> =>
  apiRequest<T>(path, {
    method: 'PATCH',
    body: body ? JSON.stringify(body) : undefined,
    ...init,
  });

export const apiUpload = async <T = unknown>(
  path: string,
  formData: FormData,
  init?: RequestInit & ApiRequestOptions,
): Promise<T> => {
  const { timeoutMs, retries, retryBaseMs, signal, ...rest } = init ?? {};
  const headers = new Headers(rest.headers);
  // The browser sets the multipart boundary itself.
  headers.delete('Content-Type');

  return performRequest<T>(path, {
    method: rest.method ?? 'POST',
    headers,
    body: rest.body ?? formData,
    action: 'Upload',
    timeoutMs: timeoutMs ?? UPLOAD_TIMEOUT_MS,
    retries: retries ?? 0,
    retryBaseMs,
    signal,
  });
};
