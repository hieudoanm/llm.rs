import { describe, expect, it } from 'vitest';
import {
  isClassificationCharts,
  isRegressionCharts,
  isClusteringCharts,
  isAnomalyCharts,
  type AnyCharts,
} from '../types';

describe('Chart type guards', () => {
  const classificationCharts = {
    classes: ['A', 'B'],
    confusionMatrix: [
      [10, 2],
      [1, 12],
    ],
    perClass: [{ class: 'A', precision: 0.9, recall: 0.85, f1: 0.87, support: 12 }],
  };

  const regressionCharts = {
    regressionPoints: { predicted: [1, 2], actual: [1.1, 1.9] },
    residuals: { predicted: [1, 2], residuals: [0.1, -0.1] },
  };

  const clusteringCharts = {
    clusterSizes: [15, 10],
    projection: { x: [1, 2], y: [1, 2], cluster: [0, 1] },
  };

  const anomalyCharts = {
    scoreHistogram: { edges: [0, 10], counts: [5], threshold: 5 },
    scorePoints: { index: [1], score: [5], anomaly: [false] },
  };

  it('isClassificationCharts returns true for classification charts', () => {
    expect(isClassificationCharts(classificationCharts as AnyCharts)).toBe(true);
  });

  it('isClassificationCharts returns false for non-classification charts', () => {
    expect(isClassificationCharts(regressionCharts as AnyCharts)).toBe(false);
    expect(isClassificationCharts(clusteringCharts as AnyCharts)).toBe(false);
    expect(isClassificationCharts(anomalyCharts as AnyCharts)).toBe(false);
  });

  it('isRegressionCharts returns true for regression charts', () => {
    expect(isRegressionCharts(regressionCharts as AnyCharts)).toBe(true);
  });

  it('isRegressionCharts returns false for non-regression charts', () => {
    expect(isRegressionCharts(classificationCharts as AnyCharts)).toBe(false);
    expect(isRegressionCharts(clusteringCharts as AnyCharts)).toBe(false);
    expect(isRegressionCharts(anomalyCharts as AnyCharts)).toBe(false);
  });

  it('isClusteringCharts returns true for clustering charts', () => {
    expect(isClusteringCharts(clusteringCharts as AnyCharts)).toBe(true);
  });

  it('isClusteringCharts returns false for non-clustering charts', () => {
    expect(isClusteringCharts(classificationCharts as AnyCharts)).toBe(false);
    expect(isClusteringCharts(regressionCharts as AnyCharts)).toBe(false);
    expect(isClusteringCharts(anomalyCharts as AnyCharts)).toBe(false);
  });

  it('isAnomalyCharts returns true for anomaly charts', () => {
    expect(isAnomalyCharts(anomalyCharts as AnyCharts)).toBe(true);
  });

  it('isAnomalyCharts returns false for non-anomaly charts', () => {
    expect(isAnomalyCharts(classificationCharts as AnyCharts)).toBe(false);
    expect(isAnomalyCharts(regressionCharts as AnyCharts)).toBe(false);
    expect(isAnomalyCharts(clusteringCharts as AnyCharts)).toBe(false);
  });
});
