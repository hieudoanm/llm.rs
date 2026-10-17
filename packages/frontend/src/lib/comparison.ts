/**
 * Comparison helpers (FEATURE-14).
 *
 * The comparison view marks the best value of every metric. The direction of
 * a metric is not always known, and a metric with an unknown direction is
 * never marked.
 */

/** The metrics where a larger value is better. */
const HIGHER_IS_BETTER = new Set([
  'accuracy',
  'precision',
  'recall',
  'f1',
  'f1_score',
  'r2',
  'roc_auc',
  'silhouette',
  'calinski_harabasz',
]);

/** The metrics where a smaller value is better. */
const LOWER_IS_BETTER = new Set([
  'rmse',
  'mse',
  'mae',
  'davies_bouldin',
  'inertia',
  'bic',
  'anomaly_rate',
]);

export type MetricDirection = 'higher' | 'lower' | 'unknown';

/** The direction of a metric, or `unknown` when it cannot be decided. */
export const metricDirection = (metric: string): MetricDirection => {
  const key = metric.toLowerCase();
  if (HIGHER_IS_BETTER.has(key)) return 'higher';
  if (LOWER_IS_BETTER.has(key)) return 'lower';
  return 'unknown';
};

export interface MetricCell {
  /** The value, or `null` when the job has no value for the metric. */
  value: number | null;
  /** True when this value is the best of the compared jobs. */
  isBest: boolean;
}

/**
 * The values of one metric across the compared jobs.
 *
 * A job without a value for the metric gets `null`, and a metric with an
 * unknown direction never marks a best value.
 */
export const metricCells = (
  metric: string,
  values: Array<number | null | undefined>,
): MetricCell[] => {
  const direction = metricDirection(metric);
  const present = values.filter((value): value is number => typeof value === 'number');
  const cells = values.map((value) => ({
    value: typeof value === 'number' ? value : null,
    isBest: false,
  }));

  if (direction === 'unknown' || present.length < 2) return cells;

  const best = direction === 'higher' ? Math.max(...present) : Math.min(...present);
  return cells.map((cell) => ({
    ...cell,
    // Only mark a best value when the metric actually differs.
    isBest: cell.value !== null && cell.value === best && present.some((v) => v !== best),
  }));
};

/** The signed spread between the best and the worst value of a metric. */
export const metricDelta = (values: Array<number | null | undefined>): number | null => {
  const present = values.filter((value): value is number => typeof value === 'number');
  if (present.length < 2) return null;
  return Math.max(...present) - Math.min(...present);
};

/**
 * The metric keys that at least two of the compared jobs carry.
 *
 * A metric that only one job has is hidden by default.
 */
export const sharedMetricKeys = (metrics: Array<Record<string, number> | null>): string[] => {
  const counts = new Map<string, number>();
  for (const metricSet of metrics) {
    for (const key of Object.keys(metricSet ?? {})) {
      counts.set(key, (counts.get(key) ?? 0) + 1);
    }
  }
  return [...counts.entries()]
    .filter(([, count]) => count >= 2)
    .map(([key]) => key)
    .sort();
};

/** Every metric key of the compared jobs, sorted. */
export const allMetricKeys = (metrics: Array<Record<string, number> | null>): string[] => {
  const keys = new Set<string>();
  for (const metricSet of metrics) {
    for (const key of Object.keys(metricSet ?? {})) keys.add(key);
  }
  return [...keys].sort();
};

/** Format a metric value for the table. */
export const formatMetric = (value: number | null): string => {
  if (value === null) return '—';
  if (!Number.isFinite(value)) return '—';
  if (Number.isInteger(value)) return String(value);
  return value.toFixed(4).replace(/0+$/, '').replace(/\.$/, '');
};
