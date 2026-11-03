'use client';

import { scaleLinear, scaleBand } from '@visx/scale';
import { Bar } from '@visx/shape';
import { AxisBottom, AxisLeft } from '@visx/axis';
import { ChartWrapper } from './ChartWrapper';

interface HistogramProps {
  title: string;
  caption: string;
  /** Histogram edges (length N+1) and counts (length N). */
  edges: number[];
  counts: number[];
  /** Optional vertical line (e.g., anomaly threshold). */
  threshold?: number | null;
  xLabel?: string;
  yLabel?: string;
}

export const Histogram = ({
  title,
  caption,
  edges,
  counts,
  threshold,
  xLabel,
  yLabel,
}: HistogramProps) => {
  if (!edges.length || !counts.length || edges.length !== counts.length + 1) {
    return (
      <ChartWrapper title={title} caption={caption}>
        <text
          x={250}
          y={150}
          textAnchor='middle'
          dominantBaseline='middle'
          className='text-text-tertiary'>
          No data
        </text>
      </ChartWrapper>
    );
  }

  const width = 500;
  const height = 300;
  const margin = { top: 20, right: 20, bottom: 60, left: 60 };
  const innerWidth = width - margin.left - margin.right;
  const innerHeight = height - margin.top - margin.bottom;

  // Bin labels for x-axis
  const binLabels = counts.map((_, i) => `${edges[i].toFixed(2)}–${edges[i + 1].toFixed(2)}`);
  const xScale = scaleBand<string>().domain(binLabels).range([0, innerWidth]).padding(0.05);
  const yScale = scaleLinear<number>()
    .domain([0, Math.max(...counts) * 1.1])
    .range([innerHeight, 0])
    .nice();

  // Threshold position in x-scale coordinates
  const thresholdX =
    threshold !== null && threshold !== undefined
      ? xScale(
          binLabels[
            Math.max(
              0,
              Math.min(
                counts.length - 1,
                counts.findIndex((_, i) => edges[i + 1] > threshold),
              ),
            )
          ],
        )
      : undefined;

  return (
    <ChartWrapper
      title={title}
      caption={caption}
      viewBox={{ width, height }}
      fallbackTable={
        <table className='table-xs table w-full max-w-md'>
          <thead>
            <tr>
              <th className='text-left'>Bin</th>
              <th className='text-right'>Count</th>
            </tr>
          </thead>
          <tbody>
            {counts.slice(0, 30).map((count, i) => (
              <tr key={i}>
                <td className='font-mono text-xs'>
                  {edges[i].toFixed(2)}–{edges[i + 1].toFixed(2)}
                </td>
                <td className='text-right'>{count}</td>
              </tr>
            ))}
            {counts.length > 30 && (
              <tr>
                <td colSpan={2} className='text-text-tertiary text-center text-xs'>
                  …and {counts.length - 30} more bins
                </td>
              </tr>
            )}
          </tbody>
        </table>
      }>
      <g transform={`translate(${margin.left}, ${margin.top})`}>
        {/* Threshold line */}
        {thresholdX !== undefined && (
          <line
            x1={thresholdX}
            y1={0}
            x2={thresholdX}
            y2={innerHeight}
            stroke='var(--color-error)'
            strokeWidth={2}
            strokeDasharray='4,4'
            opacity={0.8}
          />
        )}
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
            tickValues={xScale
              .domain()
              .filter((_, i) => i % Math.max(1, Math.ceil(binLabels.length / 10)) === 0)}
          />
        </g>
        {/* Bars */}
        <g>
          {counts.map((count, i) => (
            <Bar
              key={i}
              x={xScale(binLabels[i])!}
              y={yScale(count)}
              width={xScale.bandwidth()}
              height={innerHeight - yScale(count)}
              rx={0.5}
              ry={0.5}
              className='fill-primary transition-opacity hover:opacity-80'
            />
          ))}
        </g>
        {/* Axis labels */}
        {yLabel && (
          <text
            x={-innerHeight / 2}
            y={-40}
            transform='rotate(-90)'
            textAnchor='middle'
            className='text-text-tertiary text-xs'>
            {yLabel}
          </text>
        )}
        {xLabel && (
          <text
            x={innerWidth / 2}
            y={innerHeight + 45}
            textAnchor='middle'
            className='text-text-tertiary text-xs'>
            {xLabel}
          </text>
        )}
        {threshold !== null && threshold !== undefined && (
          <text x={thresholdX! + 4} y={12} className='text-error text-xs font-medium'>
            threshold: {threshold.toFixed(2)}
          </text>
        )}
      </g>
    </ChartWrapper>
  );
};
