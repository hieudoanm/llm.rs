import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import {
  loadApi,
  setupApiTests,
  teardownApiTests,
  jsonResponse,
  callAt,
  fetchMock,
} from '../apiTestUtils';

beforeEach(() => {
  setupApiTests();
});

afterEach(() => {
  vi.useRealTimers();
  teardownApiTests();
});

/** A fetch that never settles until its signal is aborted. */
const hangingFetch = (_url: string, init: RequestInit) =>
  new Promise<Response>((_resolve, reject) => {
    init.signal?.addEventListener('abort', () => reject(new Error('aborted')));
  });

describe('timeouts', () => {
  it('fails with REQUEST_TIMEOUT when the server does not answer', async () => {
    const { apiGet, ApiError } = await loadApi();
    fetchMock.mockImplementation(hangingFetch);

    const error = (await apiGet('/api/v1/jobs', { timeoutMs: 20, retries: 0 }).catch(
      (cause: unknown) => cause,
    )) as InstanceType<typeof ApiError>;

    expect(error).toBeInstanceOf(ApiError);
    expect(error.code).toBe('REQUEST_TIMEOUT');
    expect(error.status).toBe(0);
    expect(error.details).toEqual({ timeoutMs: 20 });
  });

  it('reports a timeout as retryable', async () => {
    const { ApiError } = await loadApi();
    const error = new ApiError('REQUEST_TIMEOUT', 'too slow', 0);

    expect(error.retryable).toBe(true);
  });
});

describe('retries', () => {
  it('retries a GET that fails with a network error', async () => {
    const { apiGet } = await loadApi();
    fetchMock.mockRejectedValueOnce(new Error('socket hang up'));
    fetchMock.mockResolvedValueOnce(jsonResponse({ jobs: [] }));

    const result = await apiGet('/api/v1/jobs', { retries: 1, retryBaseMs: 1 });

    expect(result).toEqual({ jobs: [] });
    expect(fetchMock).toHaveBeenCalledTimes(2);
  });

  it('does not retry a 400 response', async () => {
    const { apiGet } = await loadApi();
    fetchMock.mockResolvedValue(
      jsonResponse({ error: { code: 'VALIDATION_ERROR', message: 'nope' } }, 400),
    );

    await apiGet('/api/v1/jobs', { retries: 2, retryBaseMs: 1 }).catch(() => undefined);

    expect(fetchMock).toHaveBeenCalledTimes(1);
  });

  it('retries a 503 response', async () => {
    const { apiGet } = await loadApi();
    fetchMock.mockResolvedValueOnce(jsonResponse({ error: { code: 'X' } }, 503));
    fetchMock.mockResolvedValueOnce(jsonResponse({ jobs: [] }));

    await apiGet('/api/v1/jobs', { retries: 1, retryBaseMs: 1 });

    expect(fetchMock).toHaveBeenCalledTimes(2);
  });

  it('does not retry a POST without an idempotency key', async () => {
    const { apiPost } = await loadApi();
    fetchMock.mockResolvedValue(jsonResponse({ error: { code: 'X' } }, 503));

    await apiPost('/api/v1/jobs', { datasetId: 'd1' }, { retries: 2, retryBaseMs: 1 }).catch(
      () => undefined,
    );

    expect(fetchMock).toHaveBeenCalledTimes(1);
  });

  it('retries a POST that carries an idempotency key', async () => {
    const { apiPost } = await loadApi();
    fetchMock.mockResolvedValueOnce(jsonResponse({ error: { code: 'X' } }, 503));
    fetchMock.mockResolvedValueOnce(jsonResponse({ jobId: 'j1' }));

    const result = await apiPost(
      '/api/v1/jobs',
      { datasetId: 'd1' },
      { retries: 1, retryBaseMs: 1, headers: { 'Idempotency-Key': 'key-1' } },
    );

    expect(result).toEqual({ jobId: 'j1' });
    expect(fetchMock).toHaveBeenCalledTimes(2);
  });

  it('gives up after the retry budget is spent', async () => {
    const { apiGet } = await loadApi();
    fetchMock.mockRejectedValue(new Error('socket hang up'));

    const error = (await apiGet('/api/v1/jobs', { retries: 2, retryBaseMs: 1 }).catch(
      (cause: unknown) => cause,
    )) as { code: string };

    expect(error.code).toBe('NETWORK_ERROR');
    expect(fetchMock).toHaveBeenCalledTimes(3);
  });
});

describe('cancellation', () => {
  it('fails with REQUEST_ABORTED when the caller aborts', async () => {
    const { apiGet, ApiError } = await loadApi();
    const controller = new AbortController();
    fetchMock.mockImplementation(
      (_url: string, init: RequestInit) =>
        new Promise<Response>((_resolve, reject) => {
          init.signal?.addEventListener('abort', () => reject(new Error('aborted')));
        }),
    );

    const pending = apiGet('/api/v1/jobs', { signal: controller.signal, retries: 0 });
    controller.abort();

    const error = (await pending.catch((cause: unknown) => cause)) as InstanceType<typeof ApiError>;

    expect(error).toBeInstanceOf(ApiError);
    expect(error.code).toBe('REQUEST_ABORTED');
  });

  it('does not retry an aborted request', async () => {
    const { apiGet } = await loadApi();
    const controller = new AbortController();
    fetchMock.mockImplementation(
      (_url: string, init: RequestInit) =>
        new Promise<Response>((_resolve, reject) => {
          init.signal?.addEventListener('abort', () => reject(new Error('aborted')));
        }),
    );

    const pending = apiGet('/api/v1/jobs', { signal: controller.signal, retries: 3 });
    controller.abort();
    await pending.catch(() => undefined);

    expect(fetchMock).toHaveBeenCalledTimes(1);
  });
});

describe('malformed responses', () => {
  it('reports INVALID_RESPONSE when a 2xx body is not JSON', async () => {
    const { apiGet, ApiError } = await loadApi();
    fetchMock.mockResolvedValueOnce(
      new Response('{ not json', {
        status: 200,
        headers: { 'content-type': 'application/json' },
      }),
    );

    const error = (await apiGet('/api/v1/jobs', { retries: 0 }).catch(
      (cause: unknown) => cause,
    )) as InstanceType<typeof ApiError>;

    expect(error).toBeInstanceOf(ApiError);
    expect(error.code).toBe('INVALID_RESPONSE');
    expect(error.status).toBe(200);
  });
});

describe('retryDelay', () => {
  it('grows exponentially and stays under the cap', async () => {
    const { retryDelay, MAX_RETRY_DELAY_MS } = await loadApi();

    const first = retryDelay(1, 300);
    const third = retryDelay(3, 300);

    expect(first).toBeGreaterThanOrEqual(0);
    expect(first).toBeLessThan(300);
    expect(third).toBeLessThan(300 * 4);
    expect(retryDelay(20, 300)).toBeLessThan(MAX_RETRY_DELAY_MS);
  });
});

describe('apiPatch', () => {
  it('sends PATCH requests with a JSON body', async () => {
    const { apiPatch } = await loadApi();
    fetchMock.mockResolvedValueOnce(jsonResponse({ name: 'run-1' }));

    await apiPatch('/api/v1/jobs/j1', { name: 'run-1' });

    const [, init] = callAt();
    expect(init.method).toBe('PATCH');
    expect(init.body).toBe(JSON.stringify({ name: 'run-1' }));
  });
});
