'use client';

import { scaleLinear } from '@visx/scale';
import { AxisBottom, AxisLeft } from '@visx/axis';
import { ChartWrapper } from './ChartWrapper';

interface Point {
  x: number;
  y: number;
  label?: string;
  cluster?: number;
}

interface ScatterChartProps {
  title: string;
  caption: string;
  points: Point[];
  xLabel?: string;
  yLabel?: string;
  /** Reference line (e.g., y = x for predicted vs actual). */
  referenceLine?: { type: 'identity' | 'horizontal'; value?: number };
  /** Point colour by cluster/class. */
  colorByCluster?: boolean;
  /** Cluster colors. */
  clusterColors?: string[];
}

const DEFAULT_CLUSTER_COLORS = [
  'var(--color-primary)',
  'var(--color-secondary)',
  'var(--color-accent)',
  'var(--color-info)',
  'var(--color-success)',
  'var(--color-warning)',
  'var(--color-error)',
];

export const ScatterChart = ({
  title,
  caption,
  points,
  xLabel,
  yLabel,
  referenceLine,
  colorByCluster = false,
  clusterColors = DEFAULT_CLUSTER_COLORS,
}: ScatterChartProps) => {
  if (!points.length) {
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

  const xValues = points.map((p) => p.x);
  const yValues = points.map((p) => p.y);
  const xMin = Math.min(...xValues);
  const xMax = Math.max(...xValues);
  const yMin = Math.min(...yValues);
  const yMax = Math.max(...yValues);
  const xPad = (xMax - xMin) * 0.05 || 1;
  const yPad = (yMax - yMin) * 0.05 || 1;

  const xScale = scaleLinear<number>()
    .domain([xMin - xPad, xMax + xPad])
    .range([0, innerWidth])
    .nice();
  const yScale = scaleLinear<number>()
    .domain([yMin - yPad, yMax + yPad])
    .range([innerHeight, 0])
    .nice();

  return (
    <ChartWrapper
      title={title}
      caption={caption}
      viewBox={{ width, height }}
      fallbackTable={
        <table className='table-xs table w-full max-w-md'>
          <thead>
            <tr>
              <th className='text-right'>{xLabel || 'X'}</th>
              <th className='text-right'>{yLabel || 'Y'}</th>
              {colorByCluster && <th className='text-right'>Cluster</th>}
            </tr>
          </thead>
          <tbody>
            {points.slice(0, 50).map((p, i) => (
              <tr key={i}>
                <td className='text-right font-mono'>{p.x.toFixed(3)}</td>
                <td className='text-right font-mono'>{p.y.toFixed(3)}</td>
                {colorByCluster && p.cluster !== undefined && (
                  <td className='text-right'>{p.cluster}</td>
                )}
              </tr>
            ))}
            {points.length > 50 && (
              <tr>
                <td
                  colSpan={colorByCluster ? 3 : 2}
                  className='text-text-tertiary text-center text-xs'>
                  …and {points.length - 50} more points
                </td>
              </tr>
            )}
          </tbody>
        </table>
      }>
      <g transform={`translate(${margin.left}, ${margin.top})`}>
        {/* Reference line */}
        {referenceLine && (
          <g stroke='var(--color-base-content)' strokeWidth={1} strokeDasharray='4,4' opacity={0.5}>
            {referenceLine.type === 'identity' && (
              <>
                <line x1={0} y1={innerHeight} x2={innerWidth} y2={0} />
              </>
            )}
            {referenceLine.type === 'horizontal' && referenceLine.value !== undefined && (
              <>
                <line
                  x1={0}
                  y1={yScale(referenceLine.value)}
                  x2={innerWidth}
                  y2={yScale(referenceLine.value)}
                />
              </>
            )}
          </g>
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
          />
        </g>
        {/* Points */}
        <g>
          {points.map((p, i) => (
            <circle
              key={i}
              cx={xScale(p.x)}
              cy={yScale(p.y)}
              r={3}
              fill={
                colorByCluster && p.cluster !== undefined
                  ? clusterColors[p.cluster % clusterColors.length]
                  : 'var(--color-primary)'
              }
              stroke='var(--color-base-100)'
              strokeWidth={0.5}
              className='hover:r-4 transition-all'
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
      </g>
    </ChartWrapper>
  );
};
