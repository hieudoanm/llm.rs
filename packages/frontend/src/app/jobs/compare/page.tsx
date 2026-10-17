'use client';

import { Suspense } from 'react';
import { useQuery } from '@tanstack/react-query';
import Link from 'next/link';
import { useSearchParams } from 'next/navigation';
import { StatusBadge, type JobStatus } from '@/components/ui/StatusBadge';
import { ApiErrorAlert } from '@/components/ui/ApiErrorAlert';
import { ErrorBoundary } from '@/components/ui/ErrorBoundary';
import { QUERY_CACHE } from '@/lib/queryCache';
import { apiGet } from '@/lib/api';
import {
  allMetricKeys,
  formatMetric,
  metricCells,
  metricDelta,
  sharedMetricKeys,
} from '@/lib/comparison';

export const dynamic = 'force-dynamic';

interface ComparisonDataset {
  datasetId: string;
  filename: string;
  rowCount: number;
  columnCount: number;
}

interface ComparisonJob {
  jobId: string;
  name: string | null;
  tags: string[];
  status: JobStatus;
  taskType: string;
  algorithmId: string;
  createdAt: string;
  dataset: ComparisonDataset | null;
  featureColumns: string[];
  labelColumn: string | null;
  splitRatio: number;
  randomSeed: number;
  hyperparameters: Record<string, unknown>;
  metrics: Record<string, number> | null;
  artefactFormats: string[];
  error: { code: string; message: string } | null;
}

interface ComparisonWarning {
  code: string;
  message: string;
}

interface ComparisonResponse {
  jobs: ComparisonJob[];
  warnings: ComparisonWarning[];
}

const WARNING_TITLES: Record<string, string> = {
  DIFFERENT_TASKS: 'Different task types',
  DIFFERENT_DATASETS: 'Different datasets',
  DIFFERENT_FEATURES: 'Different feature columns',
  INCOMPLETE_JOBS: 'Not every job has finished',
};

const CompareContent = () => {
  const searchParams = useSearchParams();
  const ids = (searchParams.get('ids') ?? '')
    .split(',')
    .map((id) => id.trim())
    .filter((id) => id.length > 0);

  const { data, isLoading, isError, error, refetch } = useQuery<ComparisonResponse>({
    queryKey: ['job-comparison', ids.join(',')],
    queryFn: () => apiGet<ComparisonResponse>(`/api/v1/jobs/compare?ids=${ids.join(',')}`),
    enabled: ids.length >= 2,
    retry: false,
    ...QUERY_CACHE.comparison,
  });

  if (ids.length < 2) {
    return (
      <div className='mx-auto max-w-6xl px-4 pt-16 pb-12'>
        <h1 className='text-text-primary mb-4 text-xl font-semibold tracking-tight'>
          Compare jobs
        </h1>
        <div className='alert alert-info' role='status'>
          <span className='text-sm'>
            Select at least two jobs on the jobs page to compare them.
          </span>
        </div>
        <Link href='/jobs' className='btn btn-ghost mt-4'>
          Back to jobs
        </Link>
      </div>
    );
  }

  const jobs = data?.jobs ?? [];
  const metrics = jobs.map((job) => job.metrics ?? null);
  const shared = sharedMetricKeys(metrics);
  const all = allMetricKeys(metrics);

  return (
    <div className='mx-auto max-w-6xl px-4 pt-16 pb-12'>
      <div className='mb-6 flex items-center justify-between'>
        <h1 className='text-text-primary text-xl font-semibold tracking-tight'>Compare jobs</h1>
        <Link href='/jobs' className='btn btn-ghost btn-sm'>
          Back to jobs
        </Link>
      </div>

      {isError ? (
        <ApiErrorAlert error={error} onRetry={() => refetch()} />
      ) : isLoading ? (
        <div
          role='status'
          aria-label='Loading comparison'
          className='flex items-center justify-center py-20'>
          <span className='loading loading-spinner loading-lg' aria-hidden='true' />
        </div>
      ) : (
        <>
          {(data?.warnings.length ?? 0) > 0 && (
            <ul className='mb-4 flex flex-col gap-2' aria-label='Comparison warnings'>
              {(data?.warnings ?? []).map((warning) => (
                <li key={warning.code} className='alert alert-warning' role='status'>
                  <div className='flex flex-col gap-0.5'>
                    <span className='text-sm font-semibold'>
                      {WARNING_TITLES[warning.code] ?? warning.code}
                    </span>
                    <span className='text-sm'>{warning.message}</span>
                  </div>
                </li>
              ))}
            </ul>
          )}

          <div className='border-border overflow-x-auto rounded-lg border'>
            <table className='table-sm table'>
              <caption className='sr-only'>Side-by-side comparison of the selected jobs</caption>
              <thead>
                <tr>
                  <th>Attribute</th>
                  {jobs.map((job) => (
                    <th key={job.jobId}>
                      <div className='flex flex-col items-start gap-1'>
                        <span className='font-semibold'>
                          {job.name ?? `Job ${job.jobId.slice(0, 8)}`}
                        </span>
                        <StatusBadge status={job.status} />
                        <span className='font-mono text-xs'>{job.algorithmId}</span>
                        <span className='badge badge-ghost badge-xs'>{job.taskType}</span>
                        <Link href={`/jobs/${job.jobId}`} className='link text-xs'>
                          Open job
                        </Link>
                      </div>
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody>
                <tr>
                  <th scope='row'>Dataset</th>
                  {jobs.map((job) => (
                    <td key={job.jobId} className='text-sm'>
                      {job.dataset ? (
                        <>
                          <span className='block'>{job.dataset.filename}</span>
                          <span className='text-text-tertiary text-xs'>
                            {job.dataset.rowCount} rows · {job.dataset.columnCount} columns
                          </span>
                        </>
                      ) : (
                        '—'
                      )}
                    </td>
                  ))}
                </tr>
                <tr>
                  <th scope='row'>Feature columns</th>
                  {jobs.map((job) => (
                    <td key={job.jobId} className='text-sm'>
                      {job.featureColumns.length}
                    </td>
                  ))}
                </tr>
                <tr>
                  <th scope='row'>Label column</th>
                  {jobs.map((job) => (
                    <td key={job.jobId} className='font-mono text-sm'>
                      {job.labelColumn ?? '—'}
                    </td>
                  ))}
                </tr>
                <tr>
                  <th scope='row'>Split ratio</th>
                  {jobs.map((job) => (
                    <td key={job.jobId} className='text-sm'>
                      {job.splitRatio}
                    </td>
                  ))}
                </tr>
                <tr>
                  <th scope='row'>Random seed</th>
                  {jobs.map((job) => (
                    <td key={job.jobId} className='text-sm'>
                      {job.randomSeed}
                    </td>
                  ))}
                </tr>
                <tr>
                  <th scope='row'>Artefacts</th>
                  {jobs.map((job) => (
                    <td key={job.jobId} className='text-sm'>
                      {job.artefactFormats.length ? job.artefactFormats.join(', ') : '—'}
                    </td>
                  ))}
                </tr>
                {shared.map((metric) => {
                  const cells = metricCells(
                    metric,
                    jobs.map((job) => job.metrics?.[metric] ?? null),
                  );
                  const delta = metricDelta(jobs.map((job) => job.metrics?.[metric] ?? null));
                  return (
                    <tr key={metric}>
                      <th scope='row' className='font-mono text-xs'>
                        {metric}
                        {delta !== null && (
                          <span className='text-text-tertiary ml-2 font-normal'>
                            Δ {formatMetric(delta)}
                          </span>
                        )}
                      </th>
                      {cells.map((cell, index) => (
                        <td
                          key={jobs[index].jobId}
                          className={cell.isBest ? 'text-success font-semibold' : 'text-sm'}>
                          {formatMetric(cell.value)}
                          {cell.isBest && (
                            <span className='ml-1' aria-label='best value'>
                              ★
                            </span>
                          )}
                        </td>
                      ))}
                    </tr>
                  );
                })}
                {jobs.some((job) => job.error) && (
                  <tr>
                    <th scope='row'>Error</th>
                    {jobs.map((job) => (
                      <td key={job.jobId} className='text-sm'>
                        {job.error ? (
                          <span className='text-error'>
                            {job.error.code}: {job.error.message}
                          </span>
                        ) : (
                          '—'
                        )}
                      </td>
                    ))}
                  </tr>
                )}
              </tbody>
            </table>
          </div>

          {all.length > shared.length && (
            <details className='mt-4'>
              <summary className='cursor-pointer text-sm'>Show all metrics</summary>
              <ul className='mt-2 flex flex-wrap gap-2'>
                {all
                  .filter((metric) => !shared.includes(metric))
                  .map((metric) => (
                    <li key={metric} className='badge badge-ghost badge-sm'>
                      {metric}
                    </li>
                  ))}
              </ul>
            </details>
          )}
        </>
      )}
    </div>
  );
};

const Compare = () => (
  // `useSearchParams` opts the subtree out of the static prerender, so the
  // boundary keeps the build working. The error boundary keeps a render crash
  // inside the studio's own fallback.
  <ErrorBoundary
    title='This comparison could not be displayed'
    guidance='Reload the page or return to the jobs list.'>
    <Suspense
      fallback={
        <div
          role='status'
          aria-label='Loading comparison'
          className='flex items-center justify-center py-20'>
          <span className='loading loading-spinner loading-lg' aria-hidden='true' />
        </div>
      }>
      <CompareContent />
    </Suspense>
  </ErrorBoundary>
);

export default Compare;
