'use client';

import { scaleLinear } from '@visx/scale';
import { Bar } from '@visx/shape';
import { AxisBottom, AxisLeft } from '@visx/axis';
import { ChartWrapper } from './ChartWrapper';

interface SilhouettePlotProps {
  title: string;
  caption: string;
  /** Per-sample silhouette values and their cluster assignments. */
  samples: Array<{ value: number; cluster: number }>;
  /** Number of clusters. */
  nClusters: number;
}

const CLUSTER_COLORS = [
  'var(--color-primary)',
  'var(--color-secondary)',
  'var(--color-accent)',
  'var(--color-info)',
  'var(--color-success)',
  'var(--color-warning)',
  'var(--color-error)',
];

const toNumber = (v: number | { valueOf(): number }): number =>
  typeof v === 'number' ? v : v.valueOf();

export const SilhouettePlot = ({ title, caption, samples, nClusters }: SilhouettePlotProps) => {
  if (!samples.length) {
    return (
      <ChartWrapper title={title} caption={caption}>
        <text
          x={250}
          y={150}
          textAnchor='middle'
          dominantBaseline='middle'
          className='text-text-tertiary'>
          Silhouette undefined (all rows identical)
        </text>
      </ChartWrapper>
    );
  }

  const width = 600;
  const height = 300;
  const margin = { top: 20, right: 20, bottom: 60, left: 60 };
  const innerWidth = width - margin.left - margin.right;
  const innerHeight = height - margin.top - margin.bottom;

  // Group by cluster and sort by silhouette value within each cluster
  const clusters: Array<Array<{ value: number; cluster: number }>> = [];
  for (let i = 0; i < nClusters; i++) {
    clusters[i] = samples.filter((s) => s.cluster === i).sort((a, b) => a.value - b.value);
  }

  // Compute y-positions for each sample (stacked by cluster)
  let yPos = 0;
  const samplePositions: Array<{ value: number; cluster: number; y: number; height: number }> = [];
  for (let c = 0; c < nClusters; c++) {
    for (const s of clusters[c]) {
      samplePositions.push({ ...s, y: yPos, height: 1 });
      yPos += 1;
    }
    // Gap between clusters
    yPos += 3;
  }

  const yScale = scaleLinear<number>().domain([0, yPos]).range([innerHeight, 0]);
  const xScale = scaleLinear<number>().domain([-1, 1]).range([0, innerWidth]);

  // Mean silhouette line
  const meanSil = samples.reduce((a, b) => a + b.value, 0) / samples.length;

  return (
    <ChartWrapper
      title={title}
      caption={caption}
      viewBox={{ width, height }}
      fallbackTable={
        <table className='table-xs table w-full max-w-md'>
          <thead>
            <tr>
              <th className='text-right'>Cluster</th>
              <th className='text-right'>Mean Silhouette</th>
              <th className='text-right'>Size</th>
            </tr>
          </thead>
          <tbody>
            {clusters.map((cls, c) => (
              <tr key={c}>
                <td className='text-right'>{c}</td>
                <td className='text-right font-mono'>
                  {cls.length > 0
                    ? (cls.reduce((a, b) => a + b.value, 0) / cls.length).toFixed(3)
                    : 'N/A'}
                </td>
                <td className='text-right'>{cls.length}</td>
              </tr>
            ))}
            <tr>
              <td className='font-medium'>Overall</td>
              <td className='text-right font-mono font-medium'>{meanSil.toFixed(3)}</td>
              <td className='text-right font-medium'>{samples.length}</td>
            </tr>
          </tbody>
        </table>
      }>
      <g transform={`translate(${margin.left}, ${margin.top})`}>
        {/* Mean line */}
        <line
          x1={xScale(meanSil)}
          y1={0}
          x2={xScale(meanSil)}
          y2={innerHeight}
          stroke='var(--color-error)'
          strokeWidth={2}
          strokeDasharray='4,4'
          opacity={0.7}
        />
        {/* Zero line */}
        <line
          x1={xScale(0)}
          y1={0}
          x2={xScale(0)}
          y2={innerHeight}
          stroke='currentColor'
          strokeWidth={1}
          opacity={0.3}
        />
        {/* Axes */}
        <AxisLeft
          scale={yScale}
          tickStroke='currentColor'
          tickLabelProps={() => ({
            className: 'text-text-secondary text-xs',
            dx: '-0.5em',
            dy: '0.32em',
          })}
          numTicks={5}
          tickValues={clusters.map((_, c) => c)}
          tickFormat={(c) => `Cluster ${c}`}
        />
        <g transform={`translate(0, ${innerHeight})`}>
          <AxisBottom
            scale={xScale}
            tickStroke='currentColor'
            tickLabelProps={() => ({
              className: 'text-text-secondary text-xs',
              dx: '0',
              dy: '0.5em',
            })}
            numTicks={5}
            tickFormat={(v) => toNumber(v).toFixed(1)}
          />
        </g>
        {/* Bars */}
        <g>
          {samplePositions.map((s, i) => (
            <Bar
              key={i}
              x={Math.min(xScale(0), xScale(s.value))}
              y={yScale(s.y)}
              width={Math.abs(xScale(s.value) - xScale(0))}
              height={yScale(s.y) - yScale(s.y + 1)}
              fill={CLUSTER_COLORS[s.cluster % CLUSTER_COLORS.length]}
            />
          ))}
        </g>
        {/* Mean label */}
        <text x={xScale(meanSil) + 4} y={12} className='text-error text-xs font-medium'>
          mean: {meanSil.toFixed(3)}
        </text>
        <text
          x={innerWidth / 2}
          y={innerHeight + 45}
          textAnchor='middle'
          className='text-text-tertiary text-xs'>
          Silhouette value
        </text>
      </g>
    </ChartWrapper>
  );
};
