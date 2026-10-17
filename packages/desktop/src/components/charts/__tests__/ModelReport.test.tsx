import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { ModelReport } from '../ModelReport';
import { AnyCharts } from '../types';

describe('ModelReport', () => {
  const classificationCharts: AnyCharts = {
    classes: ['A', 'B'],
    confusionMatrix: [
      [10, 2],
      [1, 12],
    ],
    perClass: [
      { class: 'A', precision: 0.9, recall: 0.85, f1: 0.87, support: 12 },
      { class: 'B', precision: 0.8, recall: 0.9, f1: 0.84, support: 10 },
    ],
    selfTraining: { iterations: 3, newlyLabelled: [5, 3, 2] },
    labelCoverage: { labelled: 50, unlabelled: 30 },
  };

  const regressionCharts: AnyCharts = {
    regressionPoints: { predicted: [1, 2, 3], actual: [1.1, 1.9, 3.2] },
    residuals: { predicted: [1, 2, 3], residuals: [0.1, -0.1, 0.2] },
  };

  const clusteringCharts: AnyCharts = {
    clusterSizes: [15, 10, 8],
    projection: { x: [1, 2, 3], y: [1, 2, 3], cluster: [0, 1, 2] },
    silhouetteSamples: { value: [0.8, 0.6, 0.4], cluster: [0, 1, 2] },
  };

  const anomalyCharts: AnyCharts = {
    scoreHistogram: { edges: [0, 10, 20], counts: [5, 10], threshold: 12 },
    scorePoints: { index: [1, 2, 3], score: [5, 15, 25], anomaly: [false, true, false] },
  };

  const rlCharts: AnyCharts = {
    qValueHistogram: { edges: [0, 1, 2], counts: [4, 6] },
    tdErrorHistogram: { edges: [-1, 0, 1], counts: [3, 7] },
    fqiProgress: [
      { iteration: 1, meanTarget: 0.5 },
      { iteration: 2, meanTarget: 0.7 },
    ],
    episodeReturns: [
      { episode: 'e1', return: 1.5 },
      { episode: 'e2', return: 2.5 },
    ],
  };

  it('renders classification charts (confusion matrix + grouped bars)', () => {
    render(<ModelReport charts={classificationCharts} jobId='test-job-id' />);

    expect(screen.getByRole('heading', { name: 'Confusion Matrix' })).toBeInTheDocument();
    expect(
      screen.getByRole('heading', { name: 'Per-Class Precision, Recall, F1' }),
    ).toBeInTheDocument();
  });

  it('renders self-training charts for semi-supervised', () => {
    render(<ModelReport charts={classificationCharts} jobId='test-job-id' />);

    expect(screen.getByRole('heading', { name: 'Label Coverage' })).toBeInTheDocument();
    expect(screen.getByRole('heading', { name: 'Self-Training Progress' })).toBeInTheDocument();
  });

  it('renders regression charts', () => {
    render(<ModelReport charts={regressionCharts} jobId='test-job-id' />);

    expect(screen.getByRole('heading', { name: 'Predicted vs Actual' })).toBeInTheDocument();
    expect(screen.getByRole('heading', { name: 'Residuals vs Predicted' })).toBeInTheDocument();
    expect(screen.getByRole('heading', { name: 'Residual Distribution' })).toBeInTheDocument();
  });

  it('renders clustering charts', () => {
    render(<ModelReport charts={clusteringCharts} jobId='test-job-id' />);

    expect(screen.getByRole('heading', { name: 'Cluster Scatter (PCA-2)' })).toBeInTheDocument();
    expect(screen.getByRole('heading', { name: 'Cluster Sizes' })).toBeInTheDocument();
    expect(screen.getByRole('heading', { name: 'Silhouette Plot' })).toBeInTheDocument();
  });

  it('renders anomaly detection charts', () => {
    render(<ModelReport charts={anomalyCharts} jobId='test-job-id' />);

    expect(screen.getByRole('heading', { name: 'Anomaly Score Distribution' })).toBeInTheDocument();
    expect(screen.getByRole('heading', { name: 'Anomaly vs Normal Split' })).toBeInTheDocument();
    expect(
      screen.getByRole('heading', { name: 'Anomaly Scores by Row Order' }),
    ).toBeInTheDocument();
  });

  it('renders reinforcement learning charts (with episode returns)', () => {
    render(<ModelReport charts={rlCharts} jobId='test-job-id' />);

    expect(screen.getByRole('heading', { name: 'Q-value Distribution' })).toBeInTheDocument();
    expect(screen.getByRole('heading', { name: 'TD Error Distribution' })).toBeInTheDocument();
    expect(screen.getByRole('heading', { name: 'Mean Q per Iteration' })).toBeInTheDocument();
    expect(screen.getByRole('heading', { name: 'Episode Returns' })).toBeInTheDocument();
  });

  it('renders MLP1 training loss for a neural-net classification job', () => {
    render(
      <ModelReport
        charts={{ ...classificationCharts, mlpProgress: { epoch: [1, 2, 3], loss: [1, 0.5, 0.3] } }}
        jobId='test-job-id'
      />,
    );

    expect(screen.getByRole('heading', { name: 'Training Loss' })).toBeInTheDocument();
  });

  it('renders MLP1 training loss for a neural-net regression job', () => {
    render(
      <ModelReport
        charts={{ ...regressionCharts, mlpProgress: { epoch: [1, 2], loss: [2, 1] } }}
        jobId='test-job-id'
      />,
    );

    expect(screen.getByRole('heading', { name: 'Training Loss' })).toBeInTheDocument();
  });

  it('hides MLP1 when no loss curve is present', () => {
    render(<ModelReport charts={classificationCharts} jobId='test-job-id' />);

    expect(screen.queryByRole('heading', { name: 'Training Loss' })).not.toBeInTheDocument();
  });

  it('hides episode returns when no episode column was declared', () => {
    const withoutEpisodes = { ...rlCharts } as AnyCharts & { episodeReturns?: unknown };
    delete (withoutEpisodes as { episodeReturns?: unknown }).episodeReturns;
    render(<ModelReport charts={withoutEpisodes} jobId='test-job-id' />);

    expect(screen.getByRole('heading', { name: 'Q-value Distribution' })).toBeInTheDocument();
    expect(screen.queryByRole('heading', { name: 'Episode Returns' })).not.toBeInTheDocument();
  });

  it('renders nothing when charts is null', () => {
    render(<ModelReport charts={null} jobId='test-job-id' />);

    // Should render nothing (no headings)
    expect(screen.queryByRole('heading')).not.toBeInTheDocument();
  });

  it('renders nothing when charts is undefined', () => {
    render(<ModelReport charts={undefined} jobId='test-job-id' />);

    expect(screen.queryByRole('heading')).not.toBeInTheDocument();
  });

  it('renders nothing for unknown task type', () => {
    render(
      <ModelReport charts={{ type: 'unknown' } as unknown as AnyCharts} jobId='test-job-id' />,
    );

    expect(screen.queryByRole('heading')).not.toBeInTheDocument();
  });
});
