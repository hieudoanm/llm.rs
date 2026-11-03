'use client';

import { useQuery, useQueryClient } from '@tanstack/react-query';
import { useParams } from 'next/navigation';
import { ApiErrorAlert } from '@/components/ui/ApiErrorAlert';
import { ErrorBoundary } from '@/components/ui/ErrorBoundary';
import { JobMetadataEditor } from '@/components/jobs/JobMetadataEditor';
import { DatasetInsightsPanel } from '@/components/jobs/DatasetInsightsPanel';
import { apiPatch, ApiError } from '@/lib/api';
import { apiGetPath } from '@/lib/apiTyped';
import { QUERY_CACHE } from '@/lib/queryCache';
import { TERMINAL_STATUSES } from './_lib/constants';
import { type Job } from './_lib/types';
import { JobHeader } from './_components/JobHeader';
import { JobStepper } from './_components/JobStepper';
import { JobDetails } from './_components/JobDetails';
import { JobMetrics } from './_components/JobMetrics';
import { JobCharts } from './_components/JobCharts';
import { JobDownloads } from './_components/JobDownloads';

export const dynamic = 'force-dynamic';

const JobStatusContent = () => {
  const params = useParams();
  const jobId = typeof params === 'object' && 'jobId' in params ? (params.jobId as string) : '';
  const queryClient = useQueryClient();

  const { data, isLoading, isError, error, refetch } = useQuery<Job, Error>({
    queryKey: ['job', jobId],
    // The signal aborts the in-flight poll when the page is left.
    queryFn: ({ signal }) => apiGetPath('/api/v1/jobs/{job_id}', { job_id: jobId }, { signal }),
    refetchInterval: (query) => {
      const current = query.state.data as Job | undefined;
      return current && TERMINAL_STATUSES.includes(current.status) ? false : 1000;
    },
    retry: (failureCount, failure) => {
      if (failure instanceof ApiError && failure.status === 404) return false;
      return failureCount < 3;
    },
    enabled: !!jobId,
    // A live job is polled, so its cached copy is always stale.
    ...QUERY_CACHE.liveJob,
  });

  if (isLoading) {
    return (
      <div className='mx-auto max-w-5xl px-4 pt-16 pb-12'>
        <h1 className='text-text-primary mb-8 text-xl font-semibold tracking-tight'>Job status</h1>
        <div
          role='status'
          aria-label='Loading job'
          className='flex items-center justify-center py-20'>
          <span className='loading loading-spinner loading-lg' aria-hidden='true' />
        </div>
      </div>
    );
  }

  if (isError || !data) {
    return (
      <div className='mx-auto max-w-5xl px-4 pt-16 pb-12'>
        <h1 className='text-text-primary mb-8 text-xl font-semibold tracking-tight'>Job status</h1>
        {/* The alert carries the catalogue guidance and the "Back to jobs" action. */}
        <ApiErrorAlert error={error ?? new Error('Job not found.')} onRetry={() => refetch()} />
      </div>
    );
  }

  const isTerminal = TERMINAL_STATUSES.includes(data.status);
  const isCompleted = data.status === 'completed';
  const isFailed = data.status === 'failed';

  return (
    <div className='mx-auto max-w-5xl px-4 pt-16 pb-12'>
      <JobHeader job={data} />

      <p className='sr-only' aria-live='polite'>
        Job status: {data.status}
      </p>

      {isFailed && data.error && (
        <ApiErrorAlert
          error={
            new ApiError(
              'HTTP_ERROR',
              data.error.message,
              500,
              undefined,
              { code: data.error.code },
              data.error.code,
            )
          }
        />
      )}

      <JobStepper status={data.status} />

      <JobMetadataEditor
        jobId={jobId}
        name={data.name ?? null}
        tags={data.tags ?? []}
        onSave={async (metadata) => {
          const updated = await apiPatch<Job>(`/api/v1/jobs/${jobId}`, metadata);
          void queryClient.invalidateQueries({ queryKey: ['job', jobId] });
          return { name: updated.name ?? null, tags: updated.tags ?? [] };
        }}
      />

      <JobDetails job={data} />

      {isCompleted && data.metrics && Object.keys(data.metrics).length > 0 && (
        <JobMetrics metrics={data.metrics} jobId={jobId} />
      )}

      {isCompleted && data.charts && <JobCharts charts={data.charts} jobId={jobId} />}

      <JobDownloads artefacts={data.artefacts ?? []} jobId={jobId} isTerminal={isTerminal} />

      <DatasetInsightsPanel datasetId={data.datasetId} />

      {!isTerminal && (
        <p className='text-text-secondary text-center text-sm'>Polling for status updates…</p>
      )}
    </div>
  );
};

const JobStatusPage = () => (
  // A render crash in the detail view falls back to the studio's alert rather
  // than the framework error page.
  <ErrorBoundary
    title='This job view could not be displayed'
    guidance='Reload the page or return to the jobs list.'>
    <JobStatusContent />
  </ErrorBoundary>
);

export default JobStatusPage;
