import { TaskType, HyperparameterField } from './types';

export const TASKS: Array<{ value: TaskType; label: string; help: string }> = [
  {
    value: 'classification',
    label: 'Classification',
    help: 'Predict a class label from the selected features.',
  },
  {
    value: 'regression',
    label: 'Regression',
    help: 'Predict a numeric value. The label column must be numeric.',
  },
  {
    value: 'semi_supervised',
    label: 'Semi-supervised',
    help: 'Train on labelled rows and refine with rows whose label cell is empty.',
  },
  {
    value: 'clustering',
    label: 'Clustering',
    help: 'Group similar rows. No label column is used.',
  },
  {
    value: 'anomaly_detection',
    label: 'Anomaly detection',
    help: 'Flag unusual rows. No label column is used.',
  },
  {
    value: 'reinforcement_learning',
    label: 'Reinforcement learning',
    help: 'Fit a Q-function from logged transitions. No label column is used.',
  },
];

export const TASK_GROUPS: Array<{ label: string; tasks: TaskType[] }> = [
  { label: 'Supervised learning', tasks: ['classification', 'regression'] },
  { label: 'Semi-supervised learning', tasks: ['semi_supervised'] },
  { label: 'Unsupervised learning', tasks: ['clustering', 'anomaly_detection'] },
  { label: 'Offline reinforcement learning', tasks: ['reinforcement_learning'] },
];

export const LABEL_TASKS = new Set<TaskType>(['classification', 'regression', 'semi_supervised']);

export const RL_TASK: TaskType = 'reinforcement_learning';

export const HYPERPARAMETER_FIELDS: Record<string, HyperparameterField[]> = {
  self_training_classifier: [
    {
      key: 'threshold',
      label: 'Confidence threshold',
      min: 0.5,
      max: 1,
      step: 0.05,
      help: 'Unlabelled rows above this confidence join the training set.',
    },
    { key: 'max_iter', label: 'Maximum iterations', min: 1, step: 1 },
  ],
  kmeans: [
    {
      key: 'n_clusters',
      label: 'Number of clusters',
      min: 2,
      step: 1,
      help: 'Between 2 and rows − 1.',
    },
  ],
  mini_batch_kmeans: [
    {
      key: 'n_clusters',
      label: 'Number of clusters',
      min: 2,
      step: 1,
      help: 'Between 2 and rows − 1.',
    },
  ],
  gaussian_mixture: [
    {
      key: 'n_components',
      label: 'Number of components',
      min: 2,
      step: 1,
      help: 'Between 2 and rows − 1.',
    },
  ],
  isolation_forest: [{ key: 'n_estimators', label: 'Trees', min: 1, step: 10 }],
  fitted_q_iteration: [
    {
      key: 'n_iterations',
      label: 'Iterations',
      min: 1,
      max: 50,
      step: 1,
      help: 'FQI sweeps, 1–50. Default 10.',
    },
    {
      key: 'gamma',
      label: 'Discount γ',
      min: 0,
      max: 1,
      step: 0.05,
      help: 'Bellman discount, 0–1. Default 0.9.',
    },
    {
      key: 'max_depth',
      label: 'Tree depth',
      min: 1,
      max: 20,
      step: 1,
      help: 'Q-function tree depth, 1–20. Default 3.',
    },
    {
      key: 'learning_rate',
      label: 'Learning rate',
      min: 0.01,
      max: 1,
      step: 0.01,
      help: 'Boost learning rate, 0 < value ≤ 1. Default 0.1.',
    },
  ],
};
