'use client';

import { scaleBand, scaleLinear } from '@visx/scale';
import { Bar } from '@visx/shape';
import { AxisBottom, AxisLeft } from '@visx/axis';
import { ChartWrapper } from './ChartWrapper';
import { cn } from '@/lib/cn';

const toNumber = (v: number | { valueOf(): number }): number =>
  typeof v === 'number' ? v : v.valueOf();

interface BarChartProps {
  /** Chart title. */
  title: string;
  /** Chart caption. */
  caption: string;
  /** Array of { label, value } pairs. */
  data: Array<{ label: string; value: number }>;
  /** X-axis label. */
  xLabel?: string;
  /** Y-axis label. */
  yLabel?: string;
  /** Value format function. */
  formatValue?: (v: number) => string;
  /** Bar fill colour (semantic token class). */
  fillClass?: string;
}

export const BarChart = ({
  title,
  caption,
  data,
  xLabel,
  yLabel,
  formatValue = (v) => v.toFixed(2),
  fillClass = 'fill-primary',
}: BarChartProps) => {
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
  const margin = { top: 20, right: 20, bottom: 60, left: 60 };
  const innerWidth = width - margin.left - margin.right;
  const innerHeight = height - margin.top - margin.bottom;

  const xScale = scaleBand<string>()
    .domain(data.map((d) => d.label))
    .range([0, innerWidth])
    .padding(0.2);

  const yScale = scaleLinear<number>()
    .domain([0, Math.max(...data.map((d) => d.value)) * 1.1])
    .range([innerHeight, 0])
    .nice();

  const bars = data.map((d) => ({
    label: d.label,
    value: d.value,
    x: xScale(d.label)!,
    y: yScale(d.value),
    width: xScale.bandwidth(),
    height: innerHeight - yScale(d.value),
  }));

  return (
    <ChartWrapper
      title={title}
      caption={caption}
      viewBox={{ width, height }}
      fallbackTable={
        <table className='table-xs table w-full'>
          <thead>
            <tr>
              <th className='text-left'>{xLabel || 'Label'}</th>
              <th className='text-right'>{yLabel || 'Value'}</th>
            </tr>
          </thead>
          <tbody>
            {data.map((d) => (
              <tr key={d.label}>
                <td>{d.label}</td>
                <td className='text-right font-mono'>{formatValue(d.value)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      }>
      <g transform={`translate(${margin.left}, ${margin.top})`}>
        {/* Y axis */}
        <AxisLeft
          scale={yScale}
          tickStroke='currentColor'
          tickLabelProps={() => ({
            className: 'text-text-secondary text-xs',
            dx: '-0.5em',
            dy: '0.32em',
          })}
          numTicks={5}
          tickFormat={(v) => formatValue(toNumber(v))}
        />
        {/* X axis */}
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
        {/* Bars */}
        <g>
          {bars.map((bar) => (
            <Bar
              key={bar.label}
              x={bar.x}
              y={bar.y}
              width={bar.width}
              height={bar.height}
              rx={2}
              ry={2}
              className={cn(fillClass, 'transition-opacity hover:opacity-80')}
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
