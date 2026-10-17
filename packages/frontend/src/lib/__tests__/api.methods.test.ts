import { afterEach, beforeEach, describe, expect, it } from 'vitest';
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
  teardownApiTests();
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
