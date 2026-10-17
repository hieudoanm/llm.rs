import { render, screen, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import type { ReactNode } from 'react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const tauriGetDatasetInsights = vi.fn();
vi.mock('@/lib/tauri', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@/lib/tauri')>();
  return {
    ...actual,
    tauriGetDatasetInsights: (...args: unknown[]) => tauriGetDatasetInsights(...args),
  };
});

import { DatasetInsightsPanel } from '../DatasetInsightsPanel';

const insights = {
  datasetId: 'd1',
  sha256: 'abc',
  filename: 'data.csv',
  rowCount: 1000,
  columnCount: 2,
  totalMissingCells: 12,
  missingRate: 0.006,
  columns: [
    {
      name: 'feature1',
      inferredType: 'float64',
      missingCount: 2,
      missingRate: 0.002,
      uniqueCount: 998,
      numeric: { min: 0, max: 10, mean: 5, median: 5, std: 2 },
      topValues: [{ value: '3', count: 5 }],
      warnings: ['HIGH_MISSING_RATE'],
    },
    {
      name: 'label',
      inferredType: 'object',
      missingCount: 10,
      missingRate: 0.01,
      uniqueCount: 3,
      numeric: null,
      topValues: [],
      warnings: ['CONSTANT_COLUMN'],
    },
  ],
  warnings: [
    {
      code: 'HIGH_MISSING_RATE',
      column: 'feature1',
      message: 'feature1: more than 5% of the values are missing.',
    },
  ],
  preview: {
    offset: 0,
    limit: 2,
    totalRows: 1000,
    columns: ['feature1', 'label'],
    rows: [
      ['1', 'a'],
      ['2', 'b'],
    ],
  },
};

const wrapper = ({ children }: { children: ReactNode }) => {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
};

beforeEach(() => {
  tauriGetDatasetInsights.mockReset();
});

describe('DatasetInsightsPanel', () => {
  it('shows a loading state first', () => {
    tauriGetDatasetInsights.mockReturnValue(new Promise(() => undefined));
    render(<DatasetInsightsPanel datasetId='d1' />, { wrapper });

    expect(screen.getByRole('status', { name: 'Loading dataset insights' })).toBeInTheDocument();
  });

  it('renders the dataset summary', async () => {
    tauriGetDatasetInsights.mockResolvedValue(insights);
    render(<DatasetInsightsPanel datasetId='d1' />, { wrapper });

    await waitFor(() => expect(screen.getByText('1000')).toBeInTheDocument());
    expect(screen.getByText('Missing cells')).toBeInTheDocument();
    expect(screen.getByText('0.6%')).toBeInTheDocument();
  });

  it('renders the warnings with their suggested action', async () => {
    tauriGetDatasetInsights.mockResolvedValue(insights);
    render(<DatasetInsightsPanel datasetId='d1' />, { wrapper });

    await waitFor(() =>
      expect(
        screen.getByText('feature1: more than 5% of the values are missing.'),
      ).toBeInTheDocument(),
    );
    expect(screen.getByText('Impute or drop this column.')).toBeInTheDocument();
  });

  it('renders the column table with the notes', async () => {
    tauriGetDatasetInsights.mockResolvedValue(insights);
    render(<DatasetInsightsPanel datasetId='d1' />, { wrapper });

    await waitFor(() => expect(screen.getAllByText('feature1').length).toBeGreaterThan(0));
    expect(screen.getByText('float64')).toBeInTheDocument();
    expect(screen.getAllByText('label').length).toBeGreaterThan(0);
    expect(screen.getByText(/range 0–10, mean 5.00/)).toBeInTheDocument();
    expect(screen.getByText(/Remove this column from the feature set./)).toBeInTheDocument();
  });

  it('renders the preview rows', async () => {
    tauriGetDatasetInsights.mockResolvedValue(insights);
    render(<DatasetInsightsPanel datasetId='d1' />, { wrapper });

    await waitFor(() => expect(screen.getByText('First rows of the dataset')).toBeInTheDocument());
  });

  it('queries the insights endpoint of the dataset', async () => {
    tauriGetDatasetInsights.mockResolvedValue(insights);
    render(<DatasetInsightsPanel datasetId='d1' />, { wrapper });

    await waitFor(() => expect(tauriGetDatasetInsights).toHaveBeenCalledWith('d1'));
  });

  it('shows an error state when the insights cannot be loaded', async () => {
    tauriGetDatasetInsights.mockRejectedValue(new Error('boom'));
    render(<DatasetInsightsPanel datasetId='d1' />, { wrapper });

    await waitFor(() =>
      expect(screen.getByRole('alert')).toHaveTextContent(
        'The dataset insights could not be loaded.',
      ),
    );
  });
});
