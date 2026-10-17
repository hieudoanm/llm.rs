import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { act, renderHook, waitFor } from '@testing-library/react';
import type { ReactNode } from 'react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const push = vi.fn();
const apiPost = vi.fn();
const apiGet = vi.fn();
const apiUpload = vi.fn();

vi.mock('next/navigation', () => ({ useRouter: () => ({ push }) }));
vi.mock('@/lib/api', () => ({
  apiGet: (...args: unknown[]) => apiGet(...args),
  apiPost: (...args: unknown[]) => apiPost(...args),
  apiUpload: (...args: unknown[]) => apiUpload(...args),
}));

import { useNewJobWizard } from '../hooks';
import type { Dataset } from '../types';

const dataset: Dataset = {
  datasetId: 'd1',
  sha256: 'abc',
  filename: 'data.csv',
  byteSize: 100,
  columns: [{ name: 'feature1', inferredDtype: 'float64', nullCount: 0, sampleValues: [1] }],
  rowCount: 10,
  createdAt: '2026-10-20T00:00:00Z',
};

const wrapper = ({ children }: { children: ReactNode }) => {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
};

beforeEach(() => {
  push.mockReset();
  apiPost.mockReset();
  apiGet.mockReset();
  apiUpload.mockReset();
  apiGet.mockResolvedValue({ algorithms: [] });
  apiPost.mockResolvedValue({
    jobId: 'j1',
    status: 'queued',
    datasetId: 'd1',
    taskType: 'classification',
    algorithmId: 'logistic_regression',
    createdAt: '2026-10-20T00:00:00Z',
  });
});

describe('useNewJobWizard create-job request', () => {
  it('sends the selected export flags and an Idempotency-Key header', async () => {
    const { result } = renderHook(() => useNewJobWizard(), { wrapper });

    act(() => {
      result.current.setDataset(dataset);
      result.current.setFeatureColumns(['feature1']);
      result.current.setLabelColumn('label');
      result.current.handleAlgorithmChange('logistic_regression');
      result.current.setExport['pkl'](true);
      result.current.setExport['joblib'](true);
    });

    await act(async () => {
      await result.current.createJobMutation.mutateAsync();
    });

    const [path, body, init] = apiPost.mock.calls[0];
    expect(path).toBe('/api/v1/jobs');
    expect(body.exportPkl).toBe(true);
    expect(body.exportJoblib).toBe(true);
    // Keras is not a real export format; the request must never send it
    // (`extra='forbid'` on CreateJobRequest would reject it anyway).
    expect(body).not.toHaveProperty('exportKeras');
    expect(typeof init.headers['Idempotency-Key']).toBe('string');
  });

  it('reuses one key across retries and clears it after success', async () => {
    const { result } = renderHook(() => useNewJobWizard(), { wrapper });

    act(() => {
      result.current.setDataset(dataset);
      result.current.setFeatureColumns(['feature1']);
      result.current.setLabelColumn('label');
      result.current.handleAlgorithmChange('logistic_regression');
    });

    apiPost.mockRejectedValueOnce(new Error('network'));
    await act(async () => {
      await result.current.createJobMutation.mutateAsync().catch(() => undefined);
    });
    await act(async () => {
      await result.current.createJobMutation.mutateAsync();
    });

    const firstKey = apiPost.mock.calls[0][2].headers['Idempotency-Key'];
    const secondKey = apiPost.mock.calls[1][2].headers['Idempotency-Key'];
    expect(secondKey).toBe(firstKey);

    await waitFor(() => expect(push).toHaveBeenCalledWith('/jobs/j1'));

    // A new job after success starts with a fresh key.
    await act(async () => {
      await result.current.createJobMutation.mutateAsync();
    });
    const thirdKey = apiPost.mock.calls[2][2].headers['Idempotency-Key'];
    expect(thirdKey).not.toBe(firstKey);
  });
});
