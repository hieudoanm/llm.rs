'use client';

import { scaleLinear } from '@visx/scale';
import { LinePath } from '@visx/shape';
import { curveLinear } from 'd3-shape';
import { AxisBottom, AxisLeft } from '@visx/axis';
import { ChartWrapper } from './ChartWrapper';

interface Point {
  x: number;
  y: number;
}

interface LineChartProps {
  title: string;
  caption: string;
  /** Series data points (y values, x = index). */
  data: number[];
  /** Optional second series for comparison. */
  secondaryData?: number[];
  xLabel?: string;
  yLabel?: string;
  /** Y-axis starts at 0. */
  yZero?: boolean;
}

const toNumber = (v: number | { valueOf(): number }): number =>
  typeof v === 'number' ? v : v.valueOf();

export const LineChart = ({
  title,
  caption,
  data,
  secondaryData,
  xLabel,
  yLabel,
  yZero = true,
}: LineChartProps) => {
  if (!data.length) {
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
  const margin = { top: 20, right: 30, bottom: 60, left: 60 };
  const innerWidth = width - margin.left - margin.right;
  const innerHeight = height - margin.top - margin.bottom;

  const xScale = scaleLinear<number>()
    .domain([0, data.length - 1])
    .range([0, innerWidth]);
  const allValues = [...data, ...(secondaryData || [])];
  const yMin = yZero ? 0 : Math.min(...allValues);
  const yMax = Math.max(...allValues);
  const yPad = (yMax - yMin) * 0.1 || 1;
  const yScale = scaleLinear<number>()
    .domain([yMin - yPad, yMax + yPad])
    .range([innerHeight, 0])
    .nice();

  const lineData: Point[] = data.map((y, i) => ({ x: xScale(i), y: yScale(y) }));
  const secondaryLineData: Point[] | undefined = secondaryData?.map((y, i) => ({
    x: xScale(i),
    y: yScale(y),
  }));

  return (
    <ChartWrapper
      title={title}
      caption={caption}
      viewBox={{ width, height }}
      fallbackTable={
        <table className='table-xs table w-full max-w-md'>
          <thead>
            <tr>
              <th className='text-right'>Iteration</th>
              <th className='text-right'>Newly Labelled</th>
            </tr>
          </thead>
          <tbody>
            {data.map((y, i) => (
              <tr key={i}>
                <td className='text-right'>{i + 1}</td>
                <td className='text-right font-mono'>{y}</td>
              </tr>
            ))}
          </tbody>
        </table>
      }>
      <g transform={`translate(${margin.left}, ${margin.top})`}>
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
            tickFormat={(v) => String(toNumber(v) + 1)}
            tickValues={data
              .map((_, i) => i)
              .filter((i) => i % Math.max(1, Math.ceil(data.length / 10)) === 0)}
          />
        </g>
        {/* Primary line */}
        <LinePath
          data={lineData}
          curve={curveLinear}
          stroke='var(--color-primary)'
          strokeWidth={2}
          fill='none'
        />
        {/* Primary points */}
        <g>
          {lineData.map((p, i) => (
            <circle
              key={i}
              cx={p.x}
              cy={p.y}
              r={3}
              fill='var(--color-primary)'
              stroke='var(--color-base-100)'
              strokeWidth={1}
            />
          ))}
        </g>
        {/* Secondary line */}
        {secondaryLineData && (
          <>
            <LinePath
              data={secondaryLineData}
              curve={curveLinear}
              stroke='var(--color-secondary)'
              strokeWidth={2}
              strokeDasharray='4,4'
              fill='none'
            />
            <g>
              {secondaryLineData.map((p, i) => (
                <circle
                  key={i}
                  cx={p.x}
                  cy={p.y}
                  r={3}
                  fill='var(--color-secondary)'
                  stroke='var(--color-base-100)'
                  strokeWidth={1}
                />
              ))}
            </g>
          </>
        )}
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
