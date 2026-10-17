/** Chart data types matching the backend `charts` payload (see `docs/07-ui-ux-design/system/charts.md`). */

// Feature importance (shared across classification, regression, semi-supervised)
export interface FeatureImportanceCharts {
  featureImportance?: {
    feature: string[];
    importance: number[];
    method: string;
  };
  /** MLP1: per-epoch training loss; absent for solvers without a loss curve. */
  mlpProgress?: {
    epoch: number[];
    loss: number[];
  };
}

// Classification / semi-supervised
export interface ClassificationCharts extends FeatureImportanceCharts {
  classes: string[];
  confusionMatrix: number[][];
  perClass: Array<{
    class: string;
    precision: number;
    recall: number;
    f1: number;
    support: number;
  }>;
  selfTraining?: {
    iterations: number;
    newlyLabelled: number[];
  };
  labelCoverage?: {
    labelled: number;
    unlabelled: number;
  };
}

// Regression
export interface RegressionCharts extends FeatureImportanceCharts {
  regressionPoints: {
    predicted: number[];
    actual: number[];
  };
  residuals: {
    predicted: number[];
    residuals: number[];
  };
}

// Clustering
export interface ClusteringCharts {
  clusterSizes: number[];
  projection: {
    x: number[];
    y: number[];
    cluster: number[];
  };
  silhouetteSamples?: {
    value: number[];
    cluster: number[];
  };
}

// Anomaly detection
export interface AnomalyCharts {
  scoreHistogram: {
    edges: number[];
    counts: number[];
    threshold: number | null;
  };
  scorePoints: {
    index: number[];
    score: number[];
    anomaly: boolean[];
  };
}

// Reinforcement learning (offline Fitted Q-Iteration)
export interface ReinforcementLearningCharts {
  qValueHistogram: {
    edges: number[];
    counts: number[];
  };
  tdErrorHistogram: {
    edges: number[];
    counts: number[];
  };
  fqiProgress: Array<{
    iteration: number;
    meanTarget: number;
  }>;
  // Present only when an episode column was declared.
  episodeReturns?: Array<{
    episode: string;
    return: number;
  }>;
}

// Union
export type AnyCharts =
  | ClassificationCharts
  | RegressionCharts
  | ClusteringCharts
  | AnomalyCharts
  | ReinforcementLearningCharts;

// Helper to discriminate based on key presence (backend doesn't send a `type` field)
export const isClassificationCharts = (c: AnyCharts): c is ClassificationCharts =>
  'classes' in c && 'confusionMatrix' in c && 'perClass' in c;

export const isRegressionCharts = (c: AnyCharts): c is RegressionCharts =>
  'regressionPoints' in c && 'residuals' in c;

export const isClusteringCharts = (c: AnyCharts): c is ClusteringCharts =>
  'clusterSizes' in c && 'projection' in c;

export const isAnomalyCharts = (c: AnyCharts): c is AnomalyCharts =>
  'scoreHistogram' in c && 'scorePoints' in c;

export const isReinforcementLearningCharts = (c: AnyCharts): c is ReinforcementLearningCharts =>
  'qValueHistogram' in c && 'fqiProgress' in c;
