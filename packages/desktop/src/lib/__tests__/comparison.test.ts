import { describe, expect, it } from 'vitest';
import {
  allMetricKeys,
  formatMetric,
  metricCells,
  metricDelta,
  metricDirection,
  sharedMetricKeys,
} from '../comparison';

describe('metricDirection', () => {
  it('knows the metrics where a larger value is better', () => {
    expect(metricDirection('accuracy')).toBe('higher');
    expect(metricDirection('f1')).toBe('higher');
    expect(metricDirection('r2')).toBe('higher');
  });

  it('knows the metrics where a smaller value is better', () => {
    expect(metricDirection('rmse')).toBe('lower');
    expect(metricDirection('mae')).toBe('lower');
    expect(metricDirection('davies_bouldin')).toBe('lower');
  });

  it('does not guess for an unknown metric', () => {
    expect(metricDirection('training_seconds')).toBe('unknown');
  });

  it('ignores the case of the metric name', () => {
    expect(metricDirection('ACCURACY')).toBe('higher');
  });
});

describe('metricCells', () => {
  it('marks the best value of a higher-is-better metric', () => {
    const cells = metricCells('accuracy', [0.8, 0.95, 0.9]);

    expect(cells.map((cell) => cell.isBest)).toEqual([false, true, false]);
    expect(cells.map((cell) => cell.value)).toEqual([0.8, 0.95, 0.9]);
  });

  it('marks the best value of a lower-is-better metric', () => {
    const cells = metricCells('rmse', [0.4, 0.1, 0.3]);

    expect(cells.map((cell) => cell.isBest)).toEqual([false, true, false]);
  });

  it('never marks a value of a metric with an unknown direction', () => {
    const cells = metricCells('training_seconds', [10, 5, 1]);

    expect(cells.every((cell) => !cell.isBest)).toBe(true);
  });

  it('marks nothing when every value is the same', () => {
    const cells = metricCells('accuracy', [0.9, 0.9]);

    expect(cells.every((cell) => !cell.isBest)).toBe(true);
  });

  it('marks nothing when only one job carries the metric', () => {
    const cells = metricCells('accuracy', [0.9, null]);

    expect(cells.every((cell) => !cell.isBest)).toBe(true);
  });

  it('reports a missing value as null', () => {
    const cells = metricCells('accuracy', [0.9, undefined]);

    expect(cells[1].value).toBeNull();
  });
});

describe('metricDelta', () => {
  it('measures the spread between the best and the worst value', () => {
    expect(metricDelta([0.8, 0.95, 0.9])).toBeCloseTo(0.15);
  });

  it('returns null when fewer than two values are present', () => {
    expect(metricDelta([0.8, null])).toBeNull();
    expect(metricDelta([])).toBeNull();
  });
});

describe('sharedMetricKeys', () => {
  it('keeps the metrics that at least two jobs carry', () => {
    const shared = sharedMetricKeys([{ accuracy: 0.9, mae: 0.1 }, { accuracy: 0.8 }]);

    expect(shared).toEqual(['accuracy']);
  });

  it('ignores a job without metrics', () => {
    expect(sharedMetricKeys([{ accuracy: 0.9 }, null])).toEqual([]);
  });
});

describe('allMetricKeys', () => {
  it('lists every metric of every job', () => {
    expect(allMetricKeys([{ accuracy: 0.9 }, { mae: 0.1 }])).toEqual(['accuracy', 'mae']);
  });
});

describe('formatMetric', () => {
  it('renders a missing value as a dash', () => {
    expect(formatMetric(null)).toBe('—');
  });

  it('renders an integer without decimals', () => {
    expect(formatMetric(3)).toBe('3');
  });

  it('trims the trailing zeros of a fraction', () => {
    expect(formatMetric(0.95)).toBe('0.95');
    expect(formatMetric(0.5)).toBe('0.5');
  });

  it('renders a non-finite value as a dash', () => {
    expect(formatMetric(Number.NaN)).toBe('—');
  });
});
