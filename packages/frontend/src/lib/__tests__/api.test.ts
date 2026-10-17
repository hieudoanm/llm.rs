import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const fetchMock = vi.fn();
const originalApiUrl = process.env.NEXT_PUBLIC_API_URL;

/**
 * Import a fresh copy of the API client, optionally with `NEXT_PUBLIC_API_URL`
 * set first — `API_BASE` is derived from the environment at module load.
 */
const loadApi = async (base?: string) => {
  vi.resetModules();
  if (base === undefined) {
    delete process.env.NEXT_PUBLIC_API_URL;
  } else {
    process.env.NEXT_PUBLIC_API_URL = base;
  }
  return import('../api');
};

const jsonResponse = (body: unknown, status = 200) =>
  new Response(JSON.stringify(body), {
    status,
    headers: { 'content-type': 'application/json; charset=utf-8' },
  });

const textResponse = (body: string, status: number) =>
  new Response(body, { status, headers: { 'content-type': 'text/plain' } });

const callAt = (index = 0) => fetchMock.mock.calls[index] as [string, RequestInit];

beforeEach(() => {
  fetchMock.mockReset();
  vi.stubGlobal('fetch', fetchMock);
});

afterEach(() => {
  vi.unstubAllGlobals();
  if (originalApiUrl === undefined) {
    delete process.env.NEXT_PUBLIC_API_URL;
  } else {
    process.env.NEXT_PUBLIC_API_URL = originalApiUrl;
  }
});

describe('API_BASE', () => {
  it('defaults to the local backend', async () => {
    const { API_BASE } = await loadApi();
    expect(API_BASE).toBe('http://127.0.0.1:8000');
  });

  it('strips a trailing slash from the configured URL', async () => {
    const { API_BASE } = await loadApi('https://api.example.com/');
    expect(API_BASE).toBe('https://api.example.com');
  });
});

describe('apiRequest', () => {
  it('requests the configured base, path and JSON content type', async () => {
    const { apiRequest } = await loadApi('https://api.example.com/');
    fetchMock.mockResolvedValueOnce(jsonResponse({ ok: true }));

    const result = await apiRequest<{ ok: boolean }>('/api/v1/jobs');

    expect(result).toEqual({ ok: true });
    const [url, init] = callAt();
    expect(url).toBe('https://api.example.com/api/v1/jobs');
    expect(new Headers(init.headers).get('Content-Type')).toBe('application/json');
  });

  it('keeps an explicit content type', async () => {
    const { apiRequest } = await loadApi();
    fetchMock.mockResolvedValueOnce(jsonResponse({}));

    await apiRequest('/api/v1/jobs', { headers: { 'Content-Type': 'text/plain' } });

    const [, init] = callAt();
    expect(new Headers(init.headers).get('Content-Type')).toBe('text/plain');
  });

  it('parses non-JSON bodies as text', async () => {
    const { apiRequest } = await loadApi();
    fetchMock.mockResolvedValueOnce(textResponse('pong', 200));

    await expect(apiRequest('/health')).resolves.toBe('pong');
  });

  it('treats a missing content type as text', async () => {
    const { apiRequest } = await loadApi();
    fetchMock.mockResolvedValueOnce(new Response('ok', { status: 200 }));

    await expect(apiRequest('/health')).resolves.toBe('ok');
  });

  it('returns undefined for an empty 204 response', async () => {
    const { apiRequest } = await loadApi();
    fetchMock.mockResolvedValueOnce(new Response(null, { status: 204 }));

    await expect(apiRequest('/health')).resolves.toBeUndefined();
  });
});

describe('apiGet / apiPost / apiDelete', () => {
  it('sends GET requests', async () => {
    const { apiGet } = await loadApi();
    fetchMock.mockResolvedValueOnce(jsonResponse({ jobs: [] }));

    await apiGet('/api/v1/jobs');

    const [url, init] = callAt();
    expect(url).toBe('http://127.0.0.1:8000/api/v1/jobs');
    expect(init.method).toBe('GET');
    expect(init.body).toBeUndefined();
  });

  it('serialises the POST body as JSON', async () => {
    const { apiPost } = await loadApi();
    fetchMock.mockResolvedValueOnce(jsonResponse({ jobId: 'j1' }));

    await apiPost('/api/v1/jobs', { datasetId: 'd1' });

    const [, init] = callAt();
    expect(init.method).toBe('POST');
    expect(init.body).toBe(JSON.stringify({ datasetId: 'd1' }));
  });

  it('omits the body when none is given', async () => {
    const { apiPost } = await loadApi();
    fetchMock.mockResolvedValueOnce(jsonResponse({}));

    await apiPost('/api/v1/jobs');

    expect(callAt()[1].body).toBeUndefined();
  });

  it('sends DELETE requests', async () => {
    const { apiDelete } = await loadApi();
    fetchMock.mockResolvedValueOnce(jsonResponse({}));

    await apiDelete('/api/v1/jobs/j1');

    expect(callAt()[1].method).toBe('DELETE');
  });
});

describe('apiUpload', () => {
  it('posts multipart form data without forcing a content type', async () => {
    const { apiUpload } = await loadApi();
    fetchMock.mockResolvedValueOnce(jsonResponse({ datasetId: 'd1' }));
    const formData = new FormData();
    formData.append('file', new File(['a,b'], 'data.csv', { type: 'text/csv' }));

    const result = await apiUpload<{ datasetId: string }>('/api/v1/datasets', formData);

    expect(result).toEqual({ datasetId: 'd1' });
    const [url, init] = callAt();
    expect(url).toBe('http://127.0.0.1:8000/api/v1/datasets');
    expect(init.method).toBe('POST');
    expect(init.body).toBe(formData);
    // The browser sets the multipart boundary itself.
    expect(new Headers(init.headers).get('Content-Type')).toBeNull();
  });
});

describe('error handling', () => {
  it('maps an API error envelope onto ApiError', async () => {
    const { apiGet, ApiError } = await loadApi();
    fetchMock.mockResolvedValueOnce(
      jsonResponse(
        {
          error: {
            code: 'VALIDATION_ERROR',
            message: 'Column "price" is missing',
            requestId: 'req-1',
            details: { column: 'price' },
          },
        },
        422,
      ),
    );

    const error = await apiGet('/api/v1/jobs/j1', { retries: 0 }).catch((cause: unknown) => cause);

    expect(error).toBeInstanceOf(ApiError);
    const apiError = error as InstanceType<typeof ApiError>;
    // The transport code is `HTTP_ERROR`; the backend's own code survives in
    // `serverCode` so it is never flattened away.
    expect(apiError).toMatchObject({
      name: 'ApiError',
      code: 'HTTP_ERROR',
      serverCode: 'VALIDATION_ERROR',
      message: 'Column "price" is missing',
      status: 422,
      requestId: 'req-1',
      details: { column: 'price' },
    });
    expect(apiError).toBeInstanceOf(Error);
  });

  it('falls back to UNKNOWN_ERROR when the envelope is missing', async () => {
    const { apiGet } = await loadApi();
    fetchMock.mockResolvedValueOnce(jsonResponse({ detail: 'nope' }, 500));

    const error = (await apiGet('/api/v1/jobs', { retries: 0 }).catch(
      (cause: unknown) => cause,
    )) as {
      code: string;
      serverCode: string;
      message: string;
      status: number;
    };

    expect(error.code).toBe('HTTP_ERROR');
    expect(error.serverCode).toBe('UNKNOWN_ERROR');
    expect(error.message).toBe('Request failed with status 500');
    expect(error.status).toBe(500);
  });

  it('uses the action name in the fallback message', async () => {
    const { apiUpload } = await loadApi();
    fetchMock.mockResolvedValueOnce(textResponse('boom', 502));

    const error = (await apiUpload('/api/v1/datasets', new FormData()).catch(
      (cause: unknown) => cause,
    )) as { message: string };

    expect(error.message).toBe('Upload failed with status 502');
  });
});
