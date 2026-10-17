import { type JobStatus } from '@/components/ui/StatusBadge';

export const STEPS: JobStatus[] = [
  'queued',
  'validating',
  'training',
  'evaluating',
  'exporting',
  'completed',
];

export const TERMINAL_STATUSES: JobStatus[] = ['completed', 'failed'];
