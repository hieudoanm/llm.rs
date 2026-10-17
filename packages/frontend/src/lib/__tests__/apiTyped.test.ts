import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import {
  callAt,
  fetchMock,
  jsonResponse,
  loadApi,
  setupApiTests,
  teardownApiTests,
} from '../apiTestUtils';

const loadTypedApi = async () => {
  const api = await loadApi();
  const typed = await import('../apiTyped');
  return { api, typed };
};

beforeEach(() => {
  setupApiTests();
});

afterEach(() => {
  teardownApiTests();
});

describe('fillPath', () => {
  it('replaces every placeholder', async () => {
    const { typed } = await loadTypedApi();

    const path = typed.fillPath('/api/v1/jobs/{job_id}/artefacts/{artefact_id}', {
      job_id: 'j1',
      artefact_id: 'a2',
    });

    expect(path).toBe('/api/v1/jobs/j1/artefacts/a2');
  });

  it('URL-encodes the substituted values', async () => {
    const { typed } = await loadTypedApi();

    expect(typed.fillPath('/api/v1/jobs/{job_id}', { job_id: 'a/b' })).toBe('/api/v1/jobs/a%2Fb');
  });

  it('throws when a placeholder has no value', async () => {
    const { typed } = await loadTypedApi();

    expect(() => typed.fillPath('/api/v1/jobs/{job_id}', {})).toThrow(
      'Missing path parameter "job_id"',
    );
  });
});

describe('apiGetTyped', () => {
  it('sends a GET to the documented path', async () => {
    const { typed } = await loadTypedApi();
    fetchMock.mockResolvedValueOnce(jsonResponse({ algorithms: [] }));

    await typed.apiGetTyped('/api/v1/algorithms');

    const [url, init] = callAt();
    expect(url).toBe('http://127.0.0.1:8000/api/v1/algorithms');
    expect(init.method).toBe('GET');
  });

  it('serialises query parameters and drops empty ones', async () => {
    const { typed } = await loadTypedApi();
    fetchMock.mockResolvedValueOnce(jsonResponse({ jobs: [], hasMore: false }));

    await typed.apiGetTyped('/api/v1/jobs', {
      limit: 50,
      status: 'queued,failed',
      q: '',
    });

    expect(callAt()[0]).toBe('http://127.0.0.1:8000/api/v1/jobs?limit=50&status=queued%2Cfailed');
  });

  it('drops an empty string query parameter', async () => {
    const { typed } = await loadTypedApi();
    fetchMock.mockResolvedValueOnce(jsonResponse({ jobs: [], hasMore: false }));

    await typed.apiGetTyped('/api/v1/jobs', { limit: 50, q: '' });

    expect(callAt()[0]).toBe('http://127.0.0.1:8000/api/v1/jobs?limit=50');
  });
});

describe('apiGetPath', () => {
  it('fills the path parameters before the request', async () => {
    const { typed } = await loadTypedApi();
    fetchMock.mockResolvedValueOnce(jsonResponse({ jobId: 'j1' }));

    await typed.apiGetPath('/api/v1/jobs/{job_id}', { job_id: 'j1' });

    expect(callAt()[0]).toBe('http://127.0.0.1:8000/api/v1/jobs/j1');
  });
});

describe('apiPostTyped / apiPatchTyped / apiDeleteTyped', () => {
  it('posts the body as JSON', async () => {
    const { typed } = await loadTypedApi();
    fetchMock.mockResolvedValueOnce(jsonResponse({ jobId: 'j1' }, 202));

    await typed.apiPostTyped('/api/v1/jobs', { datasetId: 'd1' } as never);

    const [url, init] = callAt();
    expect(url).toBe('http://127.0.0.1:8000/api/v1/jobs');
    expect(init.method).toBe('POST');
    expect(init.body).toBe(JSON.stringify({ datasetId: 'd1' }));
  });

  it('patches the body as JSON', async () => {
    const { typed } = await loadTypedApi();
    fetchMock.mockResolvedValueOnce(jsonResponse({ jobId: 'j1' }));

    await typed.apiPatchTyped('/api/v1/jobs/{job_id}', { name: 'Baseline' } as never);

    const [, init] = callAt();
    expect(init.method).toBe('PATCH');
    expect(init.body).toBe(JSON.stringify({ name: 'Baseline' }));
  });

  it('deletes without a body', async () => {
    const { typed } = await loadTypedApi();
    fetchMock.mockResolvedValueOnce(new Response(null, { status: 204 }));

    await expect(typed.apiDeleteTyped('/api/v1/jobs/{job_id}')).resolves.toBeUndefined();

    const [, init] = callAt();
    expect(init.method).toBe('DELETE');
  });
});
