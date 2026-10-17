'use client';

import { BarChart } from './BarChart';
import { FeatureImportance } from './FeatureImportance';
import { GroupedBarChart } from './GroupedBarChart';
import { Heatmap } from './Heatmap';
import { ScatterChart } from './ScatterChart';
import { Histogram } from './Histogram';
import { LineChart } from './LineChart';
import { Donut } from './Donut';
import { SilhouettePlot } from './SilhouettePlot';
import { ChartDownloadMenu } from './ChartDownloadMenu';
import {
  AnyCharts,
  isClassificationCharts,
  isRegressionCharts,
  isClusteringCharts,
  isAnomalyCharts,
  isReinforcementLearningCharts,
} from './types';

interface ModelReportProps {
  charts: AnyCharts | null | undefined;
  jobId: string;
}

export const ModelReport = ({ charts, jobId }: ModelReportProps) => {
  if (!charts) {
    return null;
  }

  // Classification / semi-supervised
  if (isClassificationCharts(charts)) {
    const hasSelfTraining = !!charts.selfTraining;
    return (
      <div className='grid gap-6 sm:grid-cols-2'>
        <ChartDownloadMenu chartKey='featureImportance' jobId={jobId}>
          <FeatureImportance
            title='Feature Importance'
            caption='Top features driving the model predictions. Higher = more important.'
            data={charts.featureImportance}
          />
        </ChartDownloadMenu>
        <ChartDownloadMenu chartKey='confusionMatrix' jobId={jobId}>
          <Heatmap
            title='Confusion Matrix'
            caption='Rows = true class, columns = predicted class. Diagonal = correct predictions.'
            classes={charts.classes}
            matrix={charts.confusionMatrix}
          />
        </ChartDownloadMenu>
        <ChartDownloadMenu chartKey='perClass' jobId={jobId}>
          <GroupedBarChart
            title='Per-Class Precision, Recall, F1'
            caption='Per-class performance on the hold-out set. Higher is better.'
            data={charts.perClass}
          />
        </ChartDownloadMenu>
        {hasSelfTraining && charts.labelCoverage && (
          <ChartDownloadMenu chartKey='labelCoverage' jobId={jobId}>
            <Donut
              title='Label Coverage'
              caption='Rows with labels vs unlabelled rows used for self-training.'
              data={[
                { label: 'Labelled', value: charts.labelCoverage.labelled },
                { label: 'Unlabelled', value: charts.labelCoverage.unlabelled },
              ]}
            />
          </ChartDownloadMenu>
        )}
        {charts.selfTraining && (
          <ChartDownloadMenu chartKey='selfTraining' jobId={jobId}>
            <LineChart
              title='Self-Training Progress'
              caption='Newly labelled rows per iteration. Stops when no more rows clear the threshold.'
              data={charts.selfTraining.newlyLabelled}
              xLabel='Iteration'
              yLabel='Newly labelled rows'
            />
          </ChartDownloadMenu>
        )}
        {charts.mlpProgress && (
          <ChartDownloadMenu chartKey='mlpProgress' jobId={jobId}>
            <LineChart
              title='Training Loss'
              caption='Per-epoch loss while fitting the neural net. Should trend down and flatten.'
              data={charts.mlpProgress.loss}
              xLabel='Epoch'
              yLabel='Loss'
            />
          </ChartDownloadMenu>
        )}
      </div>
    );
  }

  // Regression
  if (isRegressionCharts(charts)) {
    // Build histogram from residuals
    const residuals = charts.residuals.residuals;
    const histBins = 20;
    const resMin = Math.min(...residuals);
    const resMax = Math.max(...residuals);
    const binWidth = (resMax - resMin) / histBins || 1;
    const edges: number[] = [];
    const counts: number[] = new Array(histBins).fill(0);
    for (let i = 0; i <= histBins; i++) {
      edges.push(resMin + i * binWidth);
    }
    for (const r of residuals) {
      const idx = Math.min(histBins - 1, Math.max(0, Math.floor((r - resMin) / binWidth)));
      counts[idx]++;
    }

    return (
      <div className='grid gap-6 sm:grid-cols-2'>
        <ChartDownloadMenu chartKey='featureImportance' jobId={jobId}>
          <FeatureImportance
            title='Feature Importance'
            caption='Top features driving the model predictions. Higher = more important.'
            data={charts.featureImportance}
          />
        </ChartDownloadMenu>
        <ChartDownloadMenu chartKey='regressionPoints' jobId={jobId}>
          <ScatterChart
            title='Predicted vs Actual'
            caption='Points on the diagonal line indicate perfect predictions.'
            points={charts.regressionPoints.predicted.map((predicted, i) => ({
              x: predicted,
              y: charts.regressionPoints.actual[i],
            }))}
            xLabel='Predicted'
            yLabel='Actual'
            referenceLine={{ type: 'identity' }}
          />
        </ChartDownloadMenu>
        <ChartDownloadMenu chartKey='residuals' jobId={jobId}>
          <ScatterChart
            title='Residuals vs Predicted'
            caption='Random scatter around zero = good fit. Patterns = systematic error.'
            points={charts.residuals.predicted.map((predicted, i) => ({
              x: predicted,
              y: charts.residuals.residuals[i],
            }))}
            xLabel='Predicted'
            yLabel='Residual'
            referenceLine={{ type: 'horizontal', value: 0 }}
          />
        </ChartDownloadMenu>
        <ChartDownloadMenu chartKey='residualDistribution' jobId={jobId}>
          <Histogram
            title='Residual Distribution'
            caption='Should be roughly bell-shaped and centred at zero.'
            edges={edges}
            counts={counts}
            xLabel='Residual'
            yLabel='Count'
          />
        </ChartDownloadMenu>
        {charts.mlpProgress && (
          <ChartDownloadMenu chartKey='mlpProgress' jobId={jobId}>
            <LineChart
              title='Training Loss'
              caption='Per-epoch loss while fitting the neural net. Should trend down and flatten.'
              data={charts.mlpProgress.loss}
              xLabel='Epoch'
              yLabel='Loss'
            />
          </ChartDownloadMenu>
        )}
      </div>
    );
  }

  // Clustering
  if (isClusteringCharts(charts)) {
    return (
      <div className='grid gap-6 sm:grid-cols-2'>
        <ChartDownloadMenu chartKey='projection' jobId={jobId}>
          <ScatterChart
            title='Cluster Scatter (PCA-2)'
            caption='2-D PCA projection coloured by cluster assignment.'
            points={charts.projection.x.map((x, i) => ({
              x,
              y: charts.projection.y[i],
              cluster: charts.projection.cluster[i],
            }))}
            xLabel='PC1'
            yLabel='PC2'
            colorByCluster={true}
          />
        </ChartDownloadMenu>
        <ChartDownloadMenu chartKey='clusterSizes' jobId={jobId}>
          <BarChart
            title='Cluster Sizes'
            caption='Number of rows assigned to each cluster.'
            data={charts.clusterSizes.map((size, i) => ({ label: `Cluster ${i}`, value: size }))}
            xLabel='Cluster'
            yLabel='Count'
          />
        </ChartDownloadMenu>
        {charts.silhouetteSamples && (
          <ChartDownloadMenu chartKey='silhouetteSamples' jobId={jobId}>
            <SilhouettePlot
              title='Silhouette Plot'
              caption='Per-sample silhouette width by cluster. Higher = better separation.'
              samples={charts.silhouetteSamples.value.map((value, i) => ({
                value,
                cluster: charts.silhouetteSamples!.cluster[i],
              }))}
              nClusters={charts.clusterSizes.length}
            />
          </ChartDownloadMenu>
        )}
      </div>
    );
  }

  // Anomaly detection
  if (isAnomalyCharts(charts)) {
    return (
      <div className='grid gap-6 sm:grid-cols-2'>
        <ChartDownloadMenu chartKey='scoreHistogram' jobId={jobId}>
          <Histogram
            title='Anomaly Score Distribution'
            caption='Histogram of decision_function scores. Threshold marks anomaly boundary.'
            edges={charts.scoreHistogram.edges}
            counts={charts.scoreHistogram.counts}
            threshold={charts.scoreHistogram.threshold}
            xLabel='Anomaly score'
            yLabel='Count'
          />
        </ChartDownloadMenu>
        <ChartDownloadMenu chartKey='scorePoints' jobId={jobId}>
          <Donut
            title='Anomaly vs Normal Split'
            caption='Share of rows flagged as anomalies.'
            data={[
              { label: 'Normal', value: charts.scorePoints.anomaly.filter((a) => !a).length },
              { label: 'Anomaly', value: charts.scorePoints.anomaly.filter((a) => a).length },
            ]}
          />
        </ChartDownloadMenu>
        <ChartDownloadMenu chartKey='scorePoints' jobId={jobId}>
          <ScatterChart
            title='Anomaly Scores by Row Order'
            caption='Anomalies (red) across the dataset. Clusters of anomalies may indicate systematic issues.'
            points={charts.scorePoints.index.map((index, i) => ({
              x: index,
              y: charts.scorePoints.score[i],
              cluster: charts.scorePoints.anomaly[i] ? 1 : 0,
            }))}
            xLabel='Row index'
            yLabel='Anomaly score'
            colorByCluster={true}
          />
        </ChartDownloadMenu>
      </div>
    );
  }

  // Reinforcement learning (offline Fitted Q-Iteration)
  if (isReinforcementLearningCharts(charts)) {
    return (
      <div className='grid gap-6 sm:grid-cols-2'>
        <ChartDownloadMenu chartKey='qValueHistogram' jobId={jobId}>
          <Histogram
            title='Q-value Distribution'
            caption='Spread of fitted Q-values across evaluation transitions.'
            edges={charts.qValueHistogram.edges}
            counts={charts.qValueHistogram.counts}
            xLabel='Q-value'
            yLabel='Count'
          />
        </ChartDownloadMenu>
        <ChartDownloadMenu chartKey='tdErrorHistogram' jobId={jobId}>
          <Histogram
            title='TD Error Distribution'
            caption='Bellman residuals; should be centred near zero and small.'
            edges={charts.tdErrorHistogram.edges}
            counts={charts.tdErrorHistogram.counts}
            xLabel='TD error'
            yLabel='Count'
          />
        </ChartDownloadMenu>
        <ChartDownloadMenu chartKey='fqiProgress' jobId={jobId}>
          <LineChart
            title='Mean Q per Iteration'
            caption='Mean fitted target per FQI sweep. Should stabilise as the fit converges.'
            data={charts.fqiProgress.map((p) => p.meanTarget)}
            xLabel='Iteration'
            yLabel='Mean target'
          />
        </ChartDownloadMenu>
        {charts.episodeReturns && (
          <ChartDownloadMenu chartKey='episodeReturns' jobId={jobId}>
            <BarChart
              title='Episode Returns'
              caption='Sum of rewards per episode in the log. Higher is better.'
              data={charts.episodeReturns.map((e) => ({ label: e.episode, value: e.return }))}
              xLabel='Episode'
              yLabel='Return'
            />
          </ChartDownloadMenu>
        )}
      </div>
    );
  }

  return null;
};
