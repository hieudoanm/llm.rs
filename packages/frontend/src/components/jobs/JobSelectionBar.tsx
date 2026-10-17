'use client';

import Link from 'next/link';
import { StatusBadge, type JobStatus } from '@/components/ui/StatusBadge';
import { PiDownload, PiX } from 'react-icons/pi';

export interface SelectedJob {
  jobId: string;
  name?: string | null;
  status: JobStatus;
  algorithmId: string;
}

interface JobSelectionBarProps {
  jobs: SelectedJob[];
  onRemove: (jobId: string) => void;
  onClear: () => void;
  /** The href of the bulk download of the selected jobs. */
  downloadHref?: string;
  /** True when at least one selected job may have artefacts. */
  hasArtefacts?: boolean;
  /** The maximum number of jobs that may be selected. */
  max?: number;
}

/**
 * The persistent selection bar of the jobs page.
 *
 * It shows the selected jobs, offers the comparison and the bulk download, and
 * explains the cap when the selection is full.
 */
export const JobSelectionBar = ({
  jobs,
  onRemove,
  onClear,
  downloadHref,
  hasArtefacts = false,
  max = 5,
}: JobSelectionBarProps) => {
  if (jobs.length === 0) return null;

  return (
    <div
      className='bg-base-200 border-border sticky bottom-0 z-20 border-t px-4 py-3 shadow-lg'
      role='region'
      aria-label='Selected jobs'>
      <div className='mx-auto flex max-w-6xl flex-wrap items-center gap-3'>
        <span className='text-text-secondary text-sm font-medium'>
          {jobs.length} of {max} selected
        </span>

        <ul className='flex flex-wrap items-center gap-2'>
          {jobs.map((job) => (
            <li key={job.jobId}>
              <span className='badge badge-outline gap-1'>
                {job.name ?? job.jobId.slice(0, 8)}
                <StatusBadge status={job.status} />
                <button
                  type='button'
                  onClick={() => onRemove(job.jobId)}
                  aria-label={`Remove ${job.name ?? job.jobId} from the selection`}>
                  <PiX className='icon-xs' aria-hidden='true' />
                </button>
              </span>
            </li>
          ))}
        </ul>

        <div className='ml-auto flex items-center gap-2'>
          {downloadHref && hasArtefacts && (
            <a href={downloadHref} className='btn btn-sm' download>
              <PiDownload className='icon-sm' aria-hidden='true' />
              Download all ({jobs.length})
            </a>
          )}
          {jobs.length >= 2 && (
            <Link
              href={`/jobs/compare?ids=${jobs.map((job) => job.jobId).join(',')}`}
              className='btn btn-primary btn-sm'>
              Compare
            </Link>
          )}
          <button type='button' onClick={onClear} className='btn btn-ghost btn-sm'>
            Clear
          </button>
        </div>
      </div>
    </div>
  );
};
