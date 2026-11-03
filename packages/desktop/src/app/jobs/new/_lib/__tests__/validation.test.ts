import { describe, expect, it } from 'vitest';
import { allIssues, isStepComplete, issuesForStep, validateWizard } from '../validation';
import type { WizardInput } from '../validation';

const base: WizardInput = {
  filename: 'data.csv',
  fileSize: 1024,
  uploaded: true,
  columns: [
    { name: 'feature1', inferredDtype: 'float64' },
    { name: 'label', inferredDtype: 'object' },
  ],
  rowCount: 100,
  featureColumns: ['feature1'],
  labelColumn: 'label',
  taskType: 'classification',
  algorithmId: 'logistic_regression',
  hyperparameters: {},
  hyperparameterFields: [],
};

describe('validateWizard', () => {
  it('accepts a complete setup', () => {
    const state = validateWizard(base);

    expect(allIssues(state)).toEqual([]);
    expect(isStepComplete(state, 'dataset')).toBe(true);
    expect(isStepComplete(state, 'mapping')).toBe(true);
    expect(isStepComplete(state, 'model')).toBe(true);
  });

  describe('dataset step', () => {
    it('asks for a file when nothing was uploaded', () => {
      const state = validateWizard({ ...base, uploaded: false, filename: null, fileSize: null });

      expect(issuesForStep(state, 'dataset')[0].message).toBe('Upload a CSV to continue.');
    });

    it('rejects a file that is not a CSV', () => {
      const state = validateWizard({
        ...base,
        uploaded: false,
        filename: 'data.xlsx',
        fileSize: 1024,
      });

      expect(issuesForStep(state, 'dataset')[0].message).toBe('Only .csv files are accepted.');
    });

    it('rejects a file above the size limit', () => {
      const state = validateWizard({
        ...base,
        uploaded: false,
        filename: 'data.csv',
        fileSize: 11 * 1024 * 1024,
      });

      expect(issuesForStep(state, 'dataset')[0].message).toBe(
        'The file is larger than the 10 MiB limit.',
      );
    });
  });

  describe('mapping step', () => {
    it('asks for at least one feature column', () => {
      const state = validateWizard({ ...base, featureColumns: [] });

      expect(issuesForStep(state, 'mapping')[0].message).toBe(
        'Select at least one feature column.',
      );
    });

    it('asks for a label column when the task needs one', () => {
      const state = validateWizard({ ...base, labelColumn: '' });

      expect(issuesForStep(state, 'mapping')[0].message).toBe('Choose a label column.');
    });

    it('reports a label column that is also a feature', () => {
      const state = validateWizard({ ...base, featureColumns: ['feature1', 'label'] });

      expect(issuesForStep(state, 'mapping').some((i) => i.message.includes('label'))).toBe(true);
    });

    it('rejects a label column for an unsupervised task', () => {
      const state = validateWizard({ ...base, taskType: 'clustering' });

      expect(issuesForStep(state, 'mapping')[0].message).toBe(
        'Clustering does not use a label column.',
      );
    });

    it('rejects a non-numeric label column for a regression', () => {
      const state = validateWizard({ ...base, taskType: 'regression' });

      expect(issuesForStep(state, 'mapping')[0].message).toBe(
        'Regression needs a numeric label column.',
      );
    });

    it('accepts a numeric label column for a regression', () => {
      const state = validateWizard({
        ...base,
        taskType: 'regression',
        columns: [
          { name: 'feature1', inferredDtype: 'float64' },
          { name: 'label', inferredDtype: 'float64' },
        ],
      });

      expect(issuesForStep(state, 'mapping')).toEqual([]);
    });

    it('rejects a label column for a reinforcement-learning task', () => {
      const state = validateWizard({ ...base, taskType: 'reinforcement_learning' });

      expect(
        issuesForStep(state, 'mapping').some((i) =>
          i.message.includes('Reinforcement learning does not use a label column.'),
        ),
      ).toBe(true);
    });
  });

  describe('model step', () => {
    it('asks for an algorithm', () => {
      const state = validateWizard({ ...base, algorithmId: '' });

      expect(issuesForStep(state, 'model')[0].message).toBe('Choose an algorithm.');
    });

    it('rejects a hyperparameter below the catalogue minimum', () => {
      const state = validateWizard({
        ...base,
        hyperparameters: { max_iter: 0 },
        hyperparameterFields: [{ key: 'max_iter', label: 'Maximum iterations', min: 1, step: 1 }],
      });

      expect(issuesForStep(state, 'model')[0].message).toBe(
        'Maximum iterations must be at least 1.',
      );
    });

    it('rejects a non-numeric hyperparameter', () => {
      const state = validateWizard({
        ...base,
        hyperparameters: { max_iter: 'many' },
        hyperparameterFields: [{ key: 'max_iter', label: 'Maximum iterations', min: 1, step: 1 }],
      });

      expect(issuesForStep(state, 'model')[0].message).toBe('Maximum iterations must be a number.');
    });

    it('rejects a fractional value for an integer hyperparameter', () => {
      const state = validateWizard({
        ...base,
        hyperparameters: { max_iter: 2.5 },
        hyperparameterFields: [{ key: 'max_iter', label: 'Maximum iterations', min: 1, step: 1 }],
      });

      expect(issuesForStep(state, 'model')[0].message).toBe(
        'Maximum iterations must be a whole number.',
      );
    });

    it('rejects a cluster count outside the range', () => {
      const state = validateWizard({
        ...base,
        hyperparameters: { n_clusters: 1 },
        hyperparameterFields: [{ key: 'n_clusters', label: 'Number of clusters', min: 2, step: 1 }],
      });

      expect(issuesForStep(state, 'model')[0].message).toBe(
        'Number of clusters must be at least 2.',
      );
    });

    it('accepts a cluster count inside the range', () => {
      const state = validateWizard({
        ...base,
        hyperparameters: { n_clusters: 3 },
        hyperparameterFields: [{ key: 'n_clusters', label: 'Number of clusters', min: 2, step: 1 }],
      });

      expect(issuesForStep(state, 'model')).toEqual([]);
    });

    it('rejects a contamination above 0.5', () => {
      const state = validateWizard({
        ...base,
        hyperparameters: { contamination: 0.8 },
        hyperparameterFields: [
          { key: 'contamination', label: 'Contamination', min: 0, step: 0.01 },
        ],
      });

      expect(issuesForStep(state, 'model')[0].message).toBe(
        'Contamination must be `auto` or between 0 and 0.5.',
      );
    });

    it('accepts the automatic contamination', () => {
      const state = validateWizard({
        ...base,
        hyperparameters: { contamination: 'auto' },
        hyperparameterFields: [
          { key: 'contamination', label: 'Contamination', min: 0, step: 0.01 },
        ],
      });

      expect(issuesForStep(state, 'model')).toEqual([]);
    });

    it('ignores a hyperparameter that was not filled in', () => {
      const state = validateWizard({
        ...base,
        hyperparameters: { max_iter: '' },
        hyperparameterFields: [{ key: 'max_iter', label: 'Maximum iterations', min: 1, step: 1 }],
      });

      expect(issuesForStep(state, 'model')).toEqual([]);
    });
  });
});
