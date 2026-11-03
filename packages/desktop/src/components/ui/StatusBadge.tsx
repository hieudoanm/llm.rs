'use client';

import { cn } from '@/lib/cn';

/**
 * The status vocabulary of the job lifecycle.
 */
export type JobStatus =
  | 'queued'
  | 'validating'
  | 'training'
  | 'evaluating'
  | 'exporting'
  | 'retrying'
  | 'completed'
  | 'failed';

const STATUS_BADGE: Record<JobStatus, string> = {
  queued: 'badge-warning',
  validating: 'badge-warning',
  training: 'badge-warning',
  evaluating: 'badge-warning',
  exporting: 'badge-warning',
  retrying: 'badge-warning',
  completed: 'badge-success',
  failed: 'badge-error',
};

const STATUS_LABEL: Record<JobStatus, string> = {
  queued: 'Waiting for worker',
  validating: 'Checking CSV and mapping',
  training: 'Fitting model',
  evaluating: 'Scoring hold-out split',
  exporting: 'Writing ONNX',
  retrying: 'Retrying after a transient error',
  completed: 'Ready to download',
  failed: 'Failed',
};

export interface StatusBadgeProps {
  status: JobStatus;
  className?: string;
  showLabel?: boolean;
}

export const StatusBadge = ({ status, className, showLabel = true }: StatusBadgeProps) => (
  <span className={cn('badge', STATUS_BADGE[status], className)}>
    {showLabel ? STATUS_LABEL[status] : status}
  </span>
);
