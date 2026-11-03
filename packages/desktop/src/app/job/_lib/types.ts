/** One downloadable file of a job, straight from the contract. */
export type Artefact = {
  artefactId: string;
  jobId: string;
  format: string;
  filename: string;
  byteSize: number;
  storageKey: string;
  createdAt: string;
};

/** The full job detail. */
export type Job = {
  id: string;
  datasetId: string;
  taskType: string;
  algorithmId: string;
  featureColumns: string[];
  labelColumn: string | null;
  rlColumns: {
    actionColumn: string;
    rewardColumn: string;
    nextStateColumns: string[];
    doneColumn: string | null;
    episodeColumn: string | null;
  } | null;
  hyperparameters: Record<string, unknown>;
  splitRatio: number;
  randomSeed: number;
  exportPkl: boolean;
  exportJoblib: boolean;
  status: string;
  errorCode: string | null;
  errorMessage: string | null;
  libraryVersions: Record<string, unknown> | null;
  name: string | null;
  tags: string[];
  attempts: number;
  maxAttempts: number;
  nextAttemptAt: string | null;
  createdAt: string;
  updatedAt: string;
  metrics: Record<string, unknown> | null;
  charts: Record<string, unknown> | null;
  artefacts: Artefact[];
};

export type JobStatus =
  | 'queued'
  | 'validating'
  | 'training'
  | 'evaluating'
  | 'exporting'
  | 'retrying'
  | 'completed'
  | 'failed';
