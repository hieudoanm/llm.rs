'use client';

import { cn } from '@/lib/cn';
import type { components } from '@/lib/apiTypes';

/**
 * The status vocabulary of the API contract.
 *
 * Derived from the generated schema rather than retyped, so a new status in
 * the OpenAPI document breaks this file at build time instead of rendering an
 * unstyled badge at runtime.
 */
export type JobStatus = components['schemas']['JobStatus'];

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
