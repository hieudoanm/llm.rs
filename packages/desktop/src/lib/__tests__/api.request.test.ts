import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import {
  loadApi,
  setupApiTests,
  teardownApiTests,
  jsonResponse,
  textResponse,
  callAt,
  fetchMock,
} from '../apiTestUtils';

beforeEach(() => {
  setupApiTests();
});

afterEach(() => {
  teardownApiTests();
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
