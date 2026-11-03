'use client';

import { Suspense, useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import Link from 'next/link';
import { usePathname, useRouter, useSearchParams } from 'next/navigation';
import { StatusBadge } from '@/components/ui/StatusBadge';
import { ApiErrorAlert } from '@/components/ui/ApiErrorAlert';
import { ErrorBoundary } from '@/components/ui/ErrorBoundary';
import type { components } from '@/lib/apiTypes';
import { JobFilterBar } from '@/components/jobs/JobFilterBar';
import { JobSelectionBar, type SelectedJob } from '@/components/jobs/JobSelectionBar';
import { JobMetadataEditor } from '@/components/jobs/JobMetadataEditor';
import { VirtualTableBody } from '@/components/jobs/VirtualTableBody';
import { API_BASE, apiPatch } from '@/lib/api';
import { apiGetTyped } from '@/lib/apiTyped';
import { MAX_COMPARE_JOBS } from '@/lib/constants';
import { QUERY_CACHE, keepPrevious } from '@/lib/queryCache';
import {
  EMPTY_FILTERS,
  fromQueryString,
  hasActiveFilters,
  readStoredFilters,
  toQueryParams,
  toQueryString,
  writeStoredFilters,
  type JobFilters,
} from '@/lib/jobs';

export const dynamic = 'force-dynamic';

/** The list response, straight from the committed contract. */
type JobList = components['schemas']['JobListResponse'];
/** One row of the list. */
type JobItem = components['schemas']['JobAcceptedResponse'];

interface JobMetadata {
  name: string | null;
  tags: string[];
}

const JobsContent = () => {
  const router = useRouter();
  const pathname = usePathname();
  const searchParams = useSearchParams();
  const queryClient = useQueryClient();
  const [editing, setEditing] = useState<string | null>(null);

  // The URL is authoritative, but a bare `/jobs` visit restores the last-used
  // filter set from localStorage (FEATURE-18). The restore runs once, after mount,
  // because localStorage is not available during the prerender.
  const restoredRef = useRef(false);
  useEffect(() => {
    if (restoredRef.current) return;
    restoredRef.current = true;
    if (searchParams.toString()) return;
    const stored = readStoredFilters();
    if (stored && hasActiveFilters(stored)) {
      router.replace(`${pathname}?${toQueryString(stored)}`, { scroll: false });
    }
  }, [pathname, router, searchParams]);

  const filters = useMemo(() => fromQueryString(searchParams.toString()), [searchParams]);
  const selection = useMemo(
    () =>
      (searchParams.get('compare') ?? '')
        .split(',')
        .map((id) => id.trim())
        .filter((id) => id.length > 0),
    [searchParams],
  );

  // Mirror the active filter set into localStorage for the next visit.
  useEffect(() => {
    writeStoredFilters(filters);
  }, [filters]);

  const updateParams = useCallback(
    (next: { filters?: JobFilters; selection?: string[] }) => {
      const params = new URLSearchParams(searchParams.toString());
      if (next.filters) {
        const query = toQueryString(next.filters);
        for (const key of [
          'status',
          'taskType',
          'algorithmId',
          'tag',
          'q',
          'createdFrom',
          'createdTo',
          'sort',
        ]) {
          params.delete(key);
        }
        if (query) {
          for (const [key, value] of new URLSearchParams(query)) params.set(key, value);
        }
      }
      if (next.selection) {
        if (next.selection.length) params.set('compare', next.selection.join(','));
        else params.delete('compare');
      }
      const search = params.toString();
      router.replace(search ? `${pathname}?${search}` : pathname, { scroll: false });
    },
    [pathname, router, searchParams],
  );

  const query = toQueryString(filters);
  const { data, isLoading, isError, error, refetch } = useQuery<JobList>({
    queryKey: ['jobs', query],
    queryFn: () => apiGetTyped('/api/v1/jobs', { limit: 50, ...toQueryParams(filters) }),
    ...QUERY_CACHE.history,
    // Changing a filter keeps the previous rows on screen while the next set
    // loads, instead of flashing an empty list.
    ...keepPrevious,
  });

  const { data: algorithmsData } = useQuery<{ algorithms: { algorithmId: string }[] }>({
    queryKey: ['algorithms', 'all'],
    queryFn: () => apiGetTyped('/api/v1/algorithms'),
    ...QUERY_CACHE.catalogue,
  });

  const metadataMutation = useMutation({
    mutationFn: ({ jobId, metadata }: { jobId: string; metadata: JobMetadata }) =>
      apiPatch<JobMetadata>(`/api/v1/jobs/${jobId}`, metadata),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ['jobs'] });
      setEditing(null);
    },
  });

  const jobs = data?.jobs ?? [];
  const algorithmIds = (algorithmsData?.algorithms ?? []).map((entry) => entry.algorithmId);
  const knownTags = [...new Set(jobs.flatMap((job) => job.tags ?? []))].sort();

  const selectedJobs: SelectedJob[] = jobs
    .filter((job) => selection.includes(job.jobId))
    .map((job) => ({
      jobId: job.jobId,
      name: job.name,
      status: job.status,
      algorithmId: job.algorithmId,
    }));

  const toggleSelection = (jobId: string) => {
    if (selection.includes(jobId)) {
      updateParams({ selection: selection.filter((id) => id !== jobId) });
      return;
    }
    if (selection.length >= MAX_COMPARE_JOBS) {
      setEditing(null);
      return;
    }
    updateParams({ selection: [...selection, jobId] });
  };

  const selectionFull = selection.length >= MAX_COMPARE_JOBS;
  const downloadHref = selection.length
    ? `${API_BASE}/api/v1/jobs/artefacts.zip?ids=${selection.join(',')}`
    : undefined;

  /** One row of the virtualised history table (FEATURE-18). */
  const renderJobRow = (job: JobItem) => (
    <>
      <td className='text-text-secondary text-sm'>{new Date(job.createdAt).toLocaleString()}</td>
      <td>
        <StatusBadge status={job.status} />
        {typeof job.queuePosition === 'number' && job.queuePosition > 0 && (
          <span className='text-text-tertiary ml-1 text-xs'>#{job.queuePosition}</span>
        )}
      </td>
      <td>
        <span className='badge badge-ghost badge-sm'>{job.taskType}</span>
      </td>
      <td className='font-mono text-sm'>{job.algorithmId}</td>
      <td>
        {job.name ? (
          <span className='text-sm'>{job.name}</span>
        ) : (
          <span className='text-text-tertiary text-sm'>—</span>
        )}
        {(job.tags ?? []).length > 0 && (
          <span className='mt-1 flex flex-wrap gap-1'>
            {(job.tags ?? []).map((tag) => (
              <span key={tag} className='badge badge-outline badge-xs'>
                {tag}
              </span>
            ))}
          </span>
        )}
      </td>
      <td>
        <div className='flex items-center gap-1'>
          <Link href={`/jobs/${job.jobId}`} className='btn btn-ghost btn-sm'>
            Open
          </Link>
          <button
            type='button'
            className='btn btn-ghost btn-sm'
            aria-pressed={selection.includes(job.jobId)}
            aria-label={`Compare ${job.name ?? job.jobId}`}
            disabled={selectionFull && !selection.includes(job.jobId)}
            onClick={() => toggleSelection(job.jobId)}>
            {selection.includes(job.jobId) ? 'Selected' : 'Compare'}
          </button>
          <a
            href={`${API_BASE}/api/v1/jobs/${job.jobId}/artefacts.zip`}
            className='btn btn-ghost btn-sm'
            download>
            Download
          </a>
          <button
            type='button'
            className='btn btn-ghost btn-sm'
            onClick={() => setEditing(editing === job.jobId ? null : job.jobId)}>
            Edit
          </button>
        </div>
      </td>
    </>
  );

  return (
    <div className='mx-auto max-w-6xl px-4 pt-16 pb-12'>
      <div className='mb-6 flex items-center justify-between'>
        <h1 className='text-text-primary text-xl font-semibold tracking-tight'>Jobs</h1>
        <Link href='/jobs/new' className='btn btn-primary btn-sm'>
          New Job
        </Link>
      </div>

      <JobFilterBar
        filters={filters}
        onChange={(next) => updateParams({ filters: next })}
        algorithmIds={algorithmIds}
        knownTags={knownTags}
      />

      {isError ? (
        <ApiErrorAlert error={error} onRetry={() => refetch()} />
      ) : isLoading || !data ? (
        <div
          role='status'
          aria-label='Loading jobs'
          className='flex items-center justify-center py-20'>
          <span className='loading loading-spinner loading-lg' aria-hidden='true' />
        </div>
      ) : jobs.length === 0 ? (
        <div className='card bg-base-100 border-border border p-12 text-center shadow'>
          <div className='mx-auto mb-4'>
            <svg
              width='48'
              height='48'
              viewBox='0 0 256 256'
              fill='none'
              stroke='currentColor'
              strokeWidth='16'
              strokeLinecap='round'
              strokeLinejoin='round'
              className='text-text-tertiary mx-auto'
              aria-hidden='true'>
              <rect x='32' y='32' width='192' height='192' rx='28' />
              <path d='M82 82h92v92' />
            </svg>
          </div>
          <h2 className='text-text-primary mb-2 text-lg font-semibold'>
            {toQueryString(filters) ? 'No jobs match these filters' : 'No jobs yet'}
          </h2>
          <p className='text-text-secondary mb-6 text-sm'>
            {toQueryString(filters)
              ? 'Try removing a filter or clearing the search.'
              : 'Train your first model to see your job history here.'}
          </p>
          {toQueryString(filters) ? (
            <button
              type='button'
              className='btn btn-primary w-full sm:w-auto'
              onClick={() => updateParams({ filters: EMPTY_FILTERS })}>
              Clear filters
            </button>
          ) : (
            <Link href='/jobs/new' className='btn btn-primary w-full sm:w-auto'>
              Train a model
            </Link>
          )}
        </div>
      ) : (
        <div className='border-border rounded-lg border'>
          <table className='table'>
            <thead>
              <tr>
                <th>Created</th>
                <th>Status</th>
                <th>Task</th>
                <th>Algorithm</th>
                <th>Name &amp; tags</th>
                <th>Actions</th>
              </tr>
            </thead>
            <VirtualTableBody
              count={jobs.length}
              renderRow={(index) => renderJobRow(jobs[index])}
              footer={
                editing && (
                  <tr>
                    <td colSpan={6}>
                      <JobMetadataEditor
                        jobId={editing}
                        name={jobs.find((job) => job.jobId === editing)?.name ?? null}
                        tags={jobs.find((job) => job.jobId === editing)?.tags ?? []}
                        onSave={(metadata) =>
                          metadataMutation.mutateAsync({ jobId: editing, metadata })
                        }
                      />
                      {metadataMutation.isError && <ApiErrorAlert error={metadataMutation.error} />}
                    </td>
                  </tr>
                )
              }
            />
          </table>
        </div>
      )}

      {selectionFull && (
        <p className='text-text-secondary mt-2 text-xs' role='status'>
          You can compare at most {MAX_COMPARE_JOBS} jobs. Remove one to select another.
        </p>
      )}

      <JobSelectionBar
        jobs={selectedJobs}
        onRemove={(jobId) => updateParams({ selection: selection.filter((id) => id !== jobId) })}
        onClear={() => updateParams({ selection: [] })}
        downloadHref={downloadHref}
        hasArtefacts={selectedJobs.some((job) => job.status === 'completed')}
      />
    </div>
  );
};

const Jobs = () => (
  // The filters live in the query string, so `useSearchParams` opts the subtree
  // out of the static prerender. The boundary keeps the build working. The
  // error boundary keeps a render crash inside the studio's own fallback.
  <ErrorBoundary
    title='The job history could not be displayed'
    guidance='Reload the page or try again.'>
    <Suspense
      fallback={
        <div
          role='status'
          aria-label='Loading jobs'
          className='flex items-center justify-center py-20'>
          <span className='loading loading-spinner loading-lg' aria-hidden='true' />
        </div>
      }>
      <JobsContent />
    </Suspense>
  </ErrorBoundary>
);

export default Jobs;
