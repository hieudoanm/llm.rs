import { describe, expect, it } from 'vitest';
import { metricInfo } from '../metrics';

const CLASSIFICATION_KEYS = ['accuracy', 'precisionMacro', 'recallMacro', 'f1Macro'];
const REGRESSION_KEYS = ['rmse', 'mae', 'r2'];
const RL_KEYS = [
  'meanReward',
  'meanQ',
  'tdErrorMean',
  'tdErrorStd',
  'episodeCount',
  'meanEpisodeReturn',
  'actions',
];
const ALL_KEYS = [...CLASSIFICATION_KEYS, ...REGRESSION_KEYS, ...RL_KEYS];

describe('metricInfo', () => {
  it.each(ALL_KEYS)('describes the %s metric', (key) => {
    const { label, description } = metricInfo(key);
    expect(label).not.toBe('');
    expect(label).not.toBe(key);
    expect(description.length).toBeGreaterThan(20);
  });

  it('uses the conventional abbreviations for regression metrics', () => {
    expect(metricInfo('rmse').label).toBe('RMSE');
    expect(metricInfo('mae').label).toBe('MAE');
    expect(metricInfo('r2').label).toBe('R²');
  });

  it('explains R² as the coefficient of determination', () => {
    expect(metricInfo('r2').description).toMatch(/coefficient of determination/i);
  });

  it('explains RMSE and MAE relative to the label units', () => {
    expect(metricInfo('rmse').description).toMatch(/units of the label column/i);
    expect(metricInfo('mae').description).toMatch(/units of the label column/i);
  });

  it('falls back to the raw key for unknown metrics', () => {
    expect(metricInfo('mysteryMetric')).toEqual({ label: 'mysteryMetric', description: '' });
  });
});
