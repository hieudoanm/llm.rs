/**
 * Plain-language descriptions for the metrics recorded by the training worker
 * (`crates/llm-core/src/training/`). Keys match the `metrics`
 * object in the job poll payload (docs/06-api-specification.md §8) and the
 * definitions in docs/02-features/FEATURE-04-Training-Job.md.
 */
export interface MetricInfo {
  label: string;
  description: string;
}

const METRIC_INFO: Record<string, MetricInfo> = {
  // Classification
  accuracy: {
    label: 'Accuracy',
    description: 'Share of test rows where the predicted class matches the true class.',
  },
  precisionMacro: {
    label: 'Precision (macro)',
    description:
      'Of the rows predicted as each class, the share that are correct — averaged equally over classes.',
  },
  recallMacro: {
    label: 'Recall (macro)',
    description:
      'Of the rows truly in each class, the share the model finds — averaged equally over classes.',
  },
  f1Macro: {
    label: 'F1 (macro)',
    description: 'Harmonic mean of macro precision and recall: one balanced score for both.',
  },
  // Regression
  rmse: {
    label: 'RMSE',
    description:
      'Root mean squared error: typical prediction error in the units of the label column, weighting large errors more.',
  },
  mae: {
    label: 'MAE',
    description:
      'Mean absolute error: average size of the prediction error, in the units of the label column.',
  },
  r2: {
    label: 'R²',
    description:
      'Coefficient of determination: share of the variance of the label explained by the model — 1 is a perfect fit, 0 is no better than predicting the mean.',
  },
  // Semi-supervised
  labelledRows: {
    label: 'Labelled rows',
    description: 'Rows whose label cell carried a real class value.',
  },
  unlabelledRows: {
    label: 'Unlabelled rows',
    description: 'Rows passed to self-training without a label.',
  },
  iterations: {
    label: 'Self-training iterations',
    description: 'Number of rounds before no additional unlabelled rows cleared the threshold.',
  },
  // Clustering
  silhouette: {
    label: 'Silhouette',
    description:
      'How well rows match their own cluster versus the next closest — higher is better, near 1.',
  },
  daviesBouldin: {
    label: 'Davies–Bouldin',
    description: 'Average similarity between clusters — lower is better, 0 is ideal.',
  },
  calinskiHarabasz: {
    label: 'Calinski–Harabasz',
    description: 'Ratio of between-cluster to within-cluster dispersion — higher is better.',
  },
  inertia: {
    label: 'Inertia',
    description: 'Sum of squared distances from rows to their cluster centroid — lower is better.',
  },
  bic: {
    label: 'BIC',
    description:
      'Bayesian information criterion for the mixture model — lower is better; it trades fit against complexity.',
  },
  // Anomaly detection
  anomalyCount: {
    label: 'Anomalies flagged',
    description: 'Rows the model labelled as outliers.',
  },
  anomalyRate: {
    label: 'Anomaly rate',
    description: 'Share of scored rows flagged as anomalies.',
  },
  scoreMean: {
    label: 'Mean anomaly score',
    description: 'Average decision score across all rows — lower means more anomalous.',
  },
  scoreStd: {
    label: 'Anomaly score std. dev.',
    description: 'Spread of decision scores across all rows.',
  },
  // Reinforcement learning (offline Fitted Q-Iteration)
  meanReward: {
    label: 'Mean reward',
    description: 'Average reward across the logged transitions supplied for training.',
  },
  meanQ: {
    label: 'Mean Q-value',
    description:
      'Average predicted value of the best action in each logged state — higher is better.',
  },
  tdErrorMean: {
    label: 'Mean TD error',
    description:
      'Average gap between the fitted Q-value and its one-step bootstrap target — closer to 0 is better.',
  },
  tdErrorStd: {
    label: 'TD error std. dev.',
    description: 'Spread of the temporal-difference errors — lower means a more consistent fit.',
  },
  episodeCount: {
    label: 'Episodes',
    description: 'Number of distinct episodes found in the logged transitions.',
  },
  meanEpisodeReturn: {
    label: 'Mean episode return',
    description: 'Average sum of rewards collected within one episode — higher is better.',
  },
  actions: {
    label: 'Actions',
    description: 'Number of distinct actions the Q-function can rank.',
  },
};

/** Label and description for a metric key; unknown keys fall back to the raw key. */
export const metricInfo = (key: string): MetricInfo =>
  METRIC_INFO[key] ?? { label: key, description: '' };
