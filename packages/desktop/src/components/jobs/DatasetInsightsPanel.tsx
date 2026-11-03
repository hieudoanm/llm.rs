'use client';

import { useQuery } from '@tanstack/react-query';
import { apiGet } from '@/lib/api';
import { QUERY_CACHE } from '@/lib/queryCache';
import { ApiErrorAlert } from '@/components/ui/ApiErrorAlert';

export interface ColumnInsight {
  name: string;
  inferredType: string;
  missingCount: number;
  missingRate: number;
  uniqueCount: number;
  numeric?: {
    min: number;
    max: number;
    mean: number;
    median: number;
    std: number;
  } | null;
  topValues?: Array<{ value: string; count: number }>;
  warnings: string[];
}

export interface DatasetWarning {
  code: string;
  column?: string | null;
  message: string;
}

export interface DatasetPreview {
  offset: number;
  limit: number;
  totalRows: number;
  columns: string[];
  rows: Array<Array<string | null>>;
}

export interface DatasetInsights {
  datasetId: string;
  sha256: string;
  filename: string;
  rowCount: number;
  columnCount: number;
  totalMissingCells: number;
  missingRate: number;
  columns: ColumnInsight[];
  warnings: DatasetWarning[];
  preview: DatasetPreview;
}

/** The action shown next to every warning of the catalogue. */
const WARNING_ACTIONS: Record<string, string> = {
  CONSTANT_COLUMN: 'Remove this column from the feature set.',
  HIGH_MISSING_RATE: 'Impute or drop this column.',
  ALL_VALUES_MISSING: 'This column carries no information.',
  HIGH_CARDINALITY: 'Looks like an identifier — keep it out of the features.',
  POSSIBLE_ID_COLUMN: 'Every row has a unique value — probably an ID.',
  MIXED_TYPES: 'Convert this column to a single type before using it.',
  TEXT_COLUMN: 'Not usable as a numeric feature.',
  SINGLE_VALUE_LABEL: 'Add more labelled rows.',
};

const formatPercent = (rate: number): string => `${(rate * 100).toFixed(1)}%`;

/** The notes shown in the "Notes" column of the column table. */
export const columnNotes = (column: ColumnInsight): string[] => {
  const notes: string[] = [];
  if (column.numeric) {
    notes.push(
      `range ${column.numeric.min}–${column.numeric.max}, mean ${column.numeric.mean.toFixed(
        2,
      )}, std ${column.numeric.std.toFixed(2)}`,
    );
  }
  if (column.topValues && column.topValues.length > 0) {
    notes.push(
      `top: ${column.topValues.map((entry) => `${entry.value} (${entry.count})`).join(', ')}`,
    );
  }
  for (const warning of column.warnings) {
    notes.push(WARNING_ACTIONS[warning] ?? warning);
  }
  return notes;
};

interface DatasetInsightsPanelProps {
  datasetId: string;
  title?: string;
}

/**
 * The dataset insights panel (FEATURE-16).
 *
 * The profile is advisory: nothing here blocks a training job.
 */
export const DatasetInsightsPanel = ({
  datasetId,
  title = 'Dataset insights',
}: DatasetInsightsPanelProps) => {
  const { data, isLoading, isError, refetch } = useQuery<DatasetInsights>({
    queryKey: ['dataset-insights', datasetId],
    queryFn: () => apiGet<DatasetInsights>(`/api/v1/datasets/${datasetId}/insights`),
    enabled: !!datasetId,
    retry: false,
    // A dataset profile is immutable once uploaded.
    ...QUERY_CACHE.insights,
  });

  if (isLoading) {
    return (
      <section className='mb-8' aria-label={title}>
        <h2 className='text-text-primary mb-4 text-lg font-semibold'>{title}</h2>
        <div role='status' aria-label='Loading dataset insights' className='py-8'>
          <span className='loading loading-spinner loading-lg' aria-hidden='true' />
        </div>
      </section>
    );
  }

  if (isError || !data) {
    return (
      <section className='mb-8' aria-label={title}>
        <h2 className='text-text-primary mb-4 text-lg font-semibold'>{title}</h2>
        <ApiErrorAlert
          error={new Error('The dataset insights could not be loaded.')}
          onRetry={() => refetch()}
        />
      </section>
    );
  }

  return (
    <section className='mb-8' aria-label={title}>
      <h2 className='text-text-primary mb-4 text-lg font-semibold'>{title}</h2>

      <dl className='mb-4 grid grid-cols-2 gap-2 sm:grid-cols-4'>
        <div className='border-border rounded-lg border p-3'>
          <dt className='text-text-tertiary text-xs'>Rows</dt>
          <dd className='text-text-primary text-lg font-semibold'>{data.rowCount}</dd>
        </div>
        <div className='border-border rounded-lg border p-3'>
          <dt className='text-text-tertiary text-xs'>Columns</dt>
          <dd className='text-text-primary text-lg font-semibold'>{data.columnCount}</dd>
        </div>
        <div className='border-border rounded-lg border p-3'>
          <dt className='text-text-tertiary text-xs'>Missing cells</dt>
          <dd className='text-text-primary text-lg font-semibold'>{data.totalMissingCells}</dd>
        </div>
        <div className='border-border rounded-lg border p-3'>
          <dt className='text-text-tertiary text-xs'>Missing rate</dt>
          <dd className='text-text-primary text-lg font-semibold'>
            {formatPercent(data.missingRate)}
          </dd>
        </div>
      </dl>

      {data.warnings.length > 0 && (
        <ul className='mb-4 flex flex-col gap-2' aria-label='Dataset warnings'>
          {data.warnings.map((warning) => {
            const action = WARNING_ACTIONS[warning.code];
            return (
              <li
                key={`${warning.code}-${warning.column ?? ''}`}
                className='alert alert-warning'
                role='status'>
                <div className='flex flex-col gap-0.5'>
                  <span className='text-sm font-semibold'>{warning.message}</span>
                  {action && <span className='text-sm'>{action}</span>}
                </div>
              </li>
            );
          })}
        </ul>
      )}

      <div className='border-border mb-4 overflow-x-auto rounded-lg border'>
        <table className='table-sm table'>
          <thead>
            <tr>
              <th>Column</th>
              <th>Type</th>
              <th>Missing</th>
              <th>Unique</th>
              <th>Notes</th>
            </tr>
          </thead>
          <tbody>
            {data.columns.map((column) => (
              <tr key={column.name}>
                <td className='font-mono text-sm'>{column.name}</td>
                <td>
                  <span className='badge badge-ghost badge-sm'>{column.inferredType}</span>
                </td>
                <td className='text-sm'>
                  {column.missingCount} ({formatPercent(column.missingRate)})
                </td>
                <td className='text-sm'>{column.uniqueCount}</td>
                <td className='text-text-secondary text-xs'>{columnNotes(column).join(' · ')}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {data.preview.rows.length > 0 && (
        <div className='border-border overflow-x-auto rounded-lg border'>
          <table className='table-xs table'>
            <caption className='sr-only'>First rows of the dataset</caption>
            <thead>
              <tr>
                {data.preview.columns.map((column) => (
                  <th key={column}>{column}</th>
                ))}
              </tr>
            </thead>
            <tbody>
              {data.preview.rows.map((row, rowIndex) => (
                <tr key={`row-${rowIndex}`}>
                  {row.map((cell, cellIndex) => (
                    <td key={`cell-${rowIndex}-${cellIndex}`} className='text-xs'>
                      {cell === '' || cell === null || cell === undefined ? '—' : cell}
                    </td>
                  ))}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </section>
  );
};
