import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import {
  WIZARD_STORAGE_KEY,
  clearWizardState,
  readWizardState,
  writeWizardState,
  type WizardState,
} from '../persistence';

const state: WizardState = {
  datasetId: 'd1',
  filename: 'data.csv',
  rowCount: 100,
  columns: [{ name: 'feature1', inferredDtype: 'float64', nullCount: 0, sampleValues: [1] }],
  featureColumns: ['feature1'],
  taskType: 'classification',
  labelColumn: 'label',
  algorithmId: 'logistic_regression',
  hyperparameters: { max_iter: 10 },
  exportPkl: true,
  exportJoblib: false,
};

beforeEach(() => {
  window.sessionStorage.clear();
});

afterEach(() => {
  window.sessionStorage.clear();
});

describe('readWizardState', () => {
  it('returns null when nothing is stored', () => {
    expect(readWizardState()).toBeNull();
  });

  it('reads the stored state back', () => {
    writeWizardState(state);

    expect(readWizardState()).toEqual(state);
  });

  it('returns null for a corrupted entry', () => {
    window.sessionStorage.setItem(WIZARD_STORAGE_KEY, '{ not json');

    expect(readWizardState()).toBeNull();
  });

  it('returns null for an entry without a dataset id', () => {
    window.sessionStorage.setItem(WIZARD_STORAGE_KEY, JSON.stringify({ filename: 'data.csv' }));

    expect(readWizardState()).toBeNull();
  });

  it('fills the defaults of a partial entry', () => {
    window.sessionStorage.setItem(
      WIZARD_STORAGE_KEY,
      JSON.stringify({ datasetId: 'd1', filename: 'data.csv' }),
    );

    const restored = readWizardState();

    expect(restored?.datasetId).toBe('d1');
    expect(restored?.taskType).toBe('classification');
    expect(restored?.featureColumns).toEqual([]);
    expect(restored?.exportPkl).toBe(false);
  });
});

describe('writeWizardState', () => {
  it('stores the state under the documented key', () => {
    writeWizardState(state);

    expect(window.sessionStorage.getItem(WIZARD_STORAGE_KEY)).toContain('d1');
  });

  it('never stores the file contents', () => {
    writeWizardState(state);

    const raw = window.sessionStorage.getItem(WIZARD_STORAGE_KEY) ?? '';
    expect(raw).not.toContain('sampleValues":[[');
  });
});

describe('clearWizardState', () => {
  it('removes the stored state', () => {
    writeWizardState(state);
    clearWizardState();

    expect(window.sessionStorage.getItem(WIZARD_STORAGE_KEY)).toBeNull();
    expect(readWizardState()).toBeNull();
  });
});
