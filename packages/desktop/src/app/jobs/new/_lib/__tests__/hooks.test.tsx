import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { act, renderHook, waitFor } from '@testing-library/react';
import type { ReactNode } from 'react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const push = vi.fn();
const tauriPostTrain = vi.fn();
const tauriGetAlgorithms = vi.fn();
const tauriPostDataset = vi.fn();

vi.mock('next/navigation', () => ({ useRouter: () => ({ push }) }));
vi.mock('@/lib/tauri', () => ({
  tauriGetAlgorithms: (...args: unknown[]) => tauriGetAlgorithms(...args),
  tauriPostDataset: (...args: unknown[]) => tauriPostDataset(...args),
  tauriPostTrain: (...args: unknown[]) => tauriPostTrain(...args),
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
  tauriPostTrain.mockReset();
  tauriGetAlgorithms.mockReset();
  tauriPostDataset.mockReset();
  tauriGetAlgorithms.mockResolvedValue({ algorithms: [] });
  tauriPostTrain.mockResolvedValue({
    jobId: 'j1',
    status: 'queued',
    datasetId: 'd1',
    taskType: 'classification',
    algorithmId: 'logistic_regression',
    createdAt: '2026-10-20T00:00:00Z',
  });
});

describe('useNewJobWizard create-job request', () => {
  it('sends the selected export flags', async () => {
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

    const [args] = tauriPostTrain.mock.calls[0];
    expect(args.request.exportPkl).toBe(true);
    expect(args.request.exportJoblib).toBe(true);
    // Keras is not a real export format; the request must never send it
    // (`extra='forbid'` on CreateJobRequest would reject it anyway).
    expect(args.request).not.toHaveProperty('exportKeras');
  });

  it('retries on failure and creates a job on success', async () => {
    const { result } = renderHook(() => useNewJobWizard(), { wrapper });

    act(() => {
      result.current.setDataset(dataset);
      result.current.setFeatureColumns(['feature1']);
      result.current.setLabelColumn('label');
      result.current.handleAlgorithmChange('logistic_regression');
    });

    tauriPostTrain.mockRejectedValueOnce(new Error('network'));
    await act(async () => {
      await result.current.createJobMutation.mutateAsync().catch(() => undefined);
    });
    await act(async () => {
      await result.current.createJobMutation.mutateAsync();
    });

    expect(tauriPostTrain).toHaveBeenCalledTimes(2);
    expect(tauriPostTrain.mock.calls[1][0].request).toHaveProperty('datasetId', 'd1');
    expect(tauriPostTrain.mock.calls[1][0].request).toHaveProperty(
      'algorithmId',
      'logistic_regression',
    );
    expect(tauriPostTrain.mock.calls[1][0].request).not.toHaveProperty('exportKeras');

    await waitFor(() => expect(push).toHaveBeenCalledWith('/jobs/j1'));
  });
});
