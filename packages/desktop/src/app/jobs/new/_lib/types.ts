export type TaskType =
  | 'classification'
  | 'regression'
  | 'semi_supervised'
  | 'clustering'
  | 'anomaly_detection'
  | 'reinforcement_learning';

export interface RlColumns {
  actionColumn: string;
  rewardColumn: string;
  nextStateColumns: string[];
  doneColumn?: string;
  episodeColumn?: string;
}

export type WizardStep = 'dataset' | 'mapping' | 'model';
export type ExportFormat = 'pkl' | 'joblib';

export interface ColumnPreview {
  name: string;
  inferredDtype: string;
  nullCount: number;
  sampleValues: unknown[];
}

export interface Dataset {
  datasetId: string;
  sha256: string;
  filename: string;
  byteSize: number;
  columns: ColumnPreview[];
  rowCount: number;
  createdAt: string;
}

export interface Algorithm {
  algorithmId: string;
  name: string;
  taskType: TaskType;
  family: string;
  description: string;
  defaultHyperparameters: Record<string, unknown>;
  exports: string[];
}

export interface JobAccepted {
  jobId: string;
  status: string;
  datasetId: string;
  taskType: TaskType;
  algorithmId: string;
  createdAt: string;
}

export interface HyperparameterField {
  key: string;
  label: string;
  min: number;
  max?: number;
  step: number;
  help?: string;
}
