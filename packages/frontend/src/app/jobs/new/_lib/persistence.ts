/**
 * Wizard state persistence (FEATURE-18).
 *
 * The in-progress setup is kept in `sessionStorage` under `mlrs:wizard:new-job`
 * so a reload does not lose the uploaded dataset. Only identifiers and
 * settings are stored — never file contents, never metric values.
 */

import type { TaskType } from './types';

export const WIZARD_STORAGE_KEY = 'mlrs:wizard:new-job';

export interface WizardState {
  /** The id of the uploaded dataset. */
  datasetId: string;
  /** The file name of the uploaded dataset. */
  filename: string;
  /** The number of rows of the uploaded dataset. */
  rowCount: number;
  /** The columns of the uploaded dataset. */
  columns: Array<{
    name: string;
    inferredDtype: string;
    nullCount: number;
    sampleValues: unknown[];
  }>;
  /** The selected feature columns. */
  featureColumns: string[];
  /** The selected task type. */
  taskType: TaskType;
  /** The selected label column. */
  labelColumn: string;
  /** The selected algorithm id. */
  algorithmId: string;
  /** The hyperparameter values. */
  hyperparameters: Record<string, string | number | boolean>;
  /** The export flags. */
  exportPkl: boolean;
  exportJoblib: boolean;
  /** The reinforcement-learning columns. */
  rlColumns?: {
    actionColumn: string;
    rewardColumn: string;
    nextStateColumns: string[];
    doneColumn?: string;
    episodeColumn?: string;
  };
}

/** Read the stored wizard state, or `null` when there is none. */
export const readWizardState = (): WizardState | null => {
  if (typeof window === 'undefined') return null;
  try {
    const raw = window.sessionStorage.getItem(WIZARD_STORAGE_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw) as Partial<WizardState>;
    if (!parsed || typeof parsed !== 'object' || !parsed.datasetId) return null;
    return {
      datasetId: parsed.datasetId,
      filename: parsed.filename ?? '',
      rowCount: parsed.rowCount ?? 0,
      columns: parsed.columns ?? [],
      featureColumns: parsed.featureColumns ?? [],
      taskType: (parsed.taskType ?? 'classification') as TaskType,
      labelColumn: parsed.labelColumn ?? '',
      algorithmId: parsed.algorithmId ?? '',
      hyperparameters: parsed.hyperparameters ?? {},
      exportPkl: parsed.exportPkl ?? false,
      exportJoblib: parsed.exportJoblib ?? false,
      rlColumns: parsed.rlColumns,
    };
  } catch {
    // A corrupted entry must never break the wizard.
    return null;
  }
};

/** Store the wizard state; a failure (private mode, quota) is not fatal. */
export const writeWizardState = (state: WizardState): void => {
  if (typeof window === 'undefined') return;
  try {
    window.sessionStorage.setItem(WIZARD_STORAGE_KEY, JSON.stringify(state));
  } catch {
    // Ignore: the wizard still works without persistence.
  }
};

/** Forget the stored wizard state. */
export const clearWizardState = (): void => {
  if (typeof window === 'undefined') return;
  try {
    window.sessionStorage.removeItem(WIZARD_STORAGE_KEY);
  } catch {
    // Ignore.
  }
};
