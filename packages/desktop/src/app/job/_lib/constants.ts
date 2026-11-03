/** Terminal job statuses. */
export const TERMINAL_STATUSES: JobStatus[] = ['completed', 'failed'];

export type JobStatus =
  | 'queued'
  | 'validating'
  | 'training'
  | 'evaluating'
  | 'exporting'
  | 'retrying'
  | 'completed'
  | 'failed';
