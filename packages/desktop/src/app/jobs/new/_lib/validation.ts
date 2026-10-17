/**
 * Live validation of the new-job wizard (FEATURE-18).
 *
 * Every rule the wizard enforces is expressed here, so the field messages,
 * the step indicators, and the summary panel all read the same result.
 */

import { MAX_UPLOAD_MB } from '@/lib/constants';
import { validateHyperparameters } from './hyperparameters';
import type { HyperparameterField, WizardStep } from './types';
import { LABEL_TASKS, RL_TASK } from './constants';

export interface ValidationIssue {
  /** The id of the offending control; the summary panel focuses it. */
  field: string;
  /** The message shown next to the control. */
  message: string;
  /** The step the issue belongs to. */
  step: WizardStep;
}

export type ValidationState = Record<WizardStep, ValidationIssue[]>;

export interface WizardInput {
  /** The name of the selected file, when one was chosen. */
  filename: string | null;
  /** The size of the selected file in bytes. */
  fileSize: number | null;
  /** True once the upload succeeded. */
  uploaded: boolean;
  /** The columns of the uploaded dataset. */
  columns: Array<{ name: string; inferredDtype: string }>;
  /** The number of rows of the uploaded dataset. */
  rowCount: number;
  /** The selected feature columns. */
  featureColumns: string[];
  /** The selected label column. */
  labelColumn: string;
  /** The selected task type. */
  taskType: string;
  /** The selected algorithm id. */
  algorithmId: string;
  /** The hyperparameter values. */
  hyperparameters: Record<string, string | number | boolean>;
  /** The catalogue range of every hyperparameter. */
  hyperparameterFields: HyperparameterField[];
}

const EMPTY: ValidationState = { dataset: [], mapping: [], model: [] };

/** The label of a task type, for the messages. */
const TASK_LABELS: Record<string, string> = {
  classification: 'Classification',
  regression: 'Regression',
  semi_supervised: 'Semi-supervised learning',
  clustering: 'Clustering',
  anomaly_detection: 'Anomaly detection',
  reinforcement_learning: 'Reinforcement learning',
};

/**
 * Validate the wizard.
 *
 * The result holds one list per step; an empty list means the step is ready.
 */
export const validateWizard = (input: WizardInput): ValidationState => {
  const state: ValidationState = { ...EMPTY, dataset: [], mapping: [], model: [] };

  // --- dataset ---------------------------------------------------------------
  if (!input.uploaded) {
    if (input.filename && !input.filename.toLowerCase().endsWith('.csv')) {
      state.dataset.push({
        field: 'dataset-file',
        step: 'dataset',
        message: 'Only .csv files are accepted.',
      });
    } else if (input.fileSize !== null && input.fileSize > MAX_UPLOAD_MB * 1024 * 1024) {
      state.dataset.push({
        field: 'dataset-file',
        step: 'dataset',
        message: `The file is larger than the ${MAX_UPLOAD_MB} MiB limit.`,
      });
    } else {
      state.dataset.push({
        field: 'dataset-file',
        step: 'dataset',
        message: 'Upload a CSV to continue.',
      });
    }
  }

  // --- mapping ---------------------------------------------------------------
  if (input.uploaded) {
    if (input.featureColumns.length === 0) {
      state.mapping.push({
        field: 'feature-columns',
        step: 'mapping',
        message: 'Select at least one feature column.',
      });
    }

    const requiresLabel = LABEL_TASKS.has(input.taskType as never);
    const usesLabel = requiresLabel || input.taskType === 'clustering';

    if (requiresLabel && !input.labelColumn) {
      state.mapping.push({
        field: 'label-column',
        step: 'mapping',
        message: 'Choose a label column.',
      });
    }

    if (!requiresLabel && input.labelColumn) {
      state.mapping.push({
        field: 'label-column',
        step: 'mapping',
        message: `${TASK_LABELS[input.taskType] ?? input.taskType} does not use a label column.`,
      });
    }

    if (input.labelColumn && input.featureColumns.includes(input.labelColumn)) {
      state.mapping.push({
        field: 'label-column',
        step: 'mapping',
        message: `${input.labelColumn} is used as both the label and a feature.`,
      });
    }

    if (input.taskType === 'regression' && input.labelColumn) {
      const label = input.columns.find((column) => column.name === input.labelColumn);
      const dtype = (label?.inferredDtype ?? '').toLowerCase();
      if (dtype && !dtype.includes('int') && !dtype.includes('float') && !dtype.includes('num')) {
        state.mapping.push({
          field: 'label-column',
          step: 'mapping',
          message: 'Regression needs a numeric label column.',
        });
      }
    }

    if (usesLabel && input.taskType === RL_TASK) {
      state.mapping.push({
        field: 'label-column',
        step: 'mapping',
        message: 'Reinforcement learning does not use a label column.',
      });
    }
  }

  // --- model -----------------------------------------------------------------
  if (input.uploaded) {
    if (!input.algorithmId) {
      state.model.push({
        field: 'algorithm',
        step: 'model',
        message: 'Choose an algorithm.',
      });
    }

    // The rules themselves live in the Zod schema (`hyperparameters.ts`), so
    // the form's inline messages and this summary cannot drift apart.
    for (const error of validateHyperparameters(
      input.hyperparameterFields,
      input.hyperparameters,
      input.rowCount,
    )) {
      state.model.push({
        field: `hyperparameter-${error.key}`,
        step: 'model',
        message: error.message,
      });
    }
  }

  return state;
};

/** Every issue of the wizard, in step order. */
export const allIssues = (state: ValidationState): ValidationIssue[] => [
  ...state.dataset,
  ...state.mapping,
  ...state.model,
];

/** The issues of one step. */
export const issuesForStep = (state: ValidationState, step: WizardStep): ValidationIssue[] =>
  state[step] ?? [];

/** True when a step has no issue left. */
export const isStepComplete = (state: ValidationState, step: WizardStep): boolean =>
  issuesForStep(state, step).length === 0;
