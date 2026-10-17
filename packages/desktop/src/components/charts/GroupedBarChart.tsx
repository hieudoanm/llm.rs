'use client';

import { scaleBand, scaleLinear } from '@visx/scale';
import { Bar } from '@visx/shape';
import { AxisBottom, AxisLeft } from '@visx/axis';
import { ChartWrapper } from './ChartWrapper';
import { cn } from '@/lib/cn';

interface GroupedBarChartProps {
  title: string;
  caption: string;
  /** Array of { class, precision, recall, f1, support }. */
  data: Array<{ class: string; precision: number; recall: number; f1: number; support: number }>;
  /** Y-axis max (default 1.0 for classification metrics). */
  yMax?: number;
}

const METRICS = [
  { key: 'precision', label: 'Precision', fill: 'fill-primary' },
  { key: 'recall', label: 'Recall', fill: 'fill-secondary' },
  { key: 'f1', label: 'F1', fill: 'fill-accent' },
] as const;

export const GroupedBarChart = ({ title, caption, data, yMax = 1.0 }: GroupedBarChartProps) => {
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

  const width = 600;
  const height = 300;
  const margin = { top: 20, right: 30, bottom: 60, left: 60 };
  const innerWidth = width - margin.left - margin.right;
  const innerHeight = height - margin.top - margin.bottom;

  const classes = data.map((d) => d.class);
  const xScale = scaleBand<string>().domain(classes).range([0, innerWidth]).padding(0.2);
  const xScaleMetric = scaleBand<string>()
    .domain(METRICS.map((m) => m.key))
    .range([0, xScale.bandwidth()])
    .padding(0.05);

  const yScale = scaleLinear<number>().domain([0, yMax]).range([innerHeight, 0]).nice();

  return (
    <ChartWrapper
      title={title}
      caption={caption}
      viewBox={{ width, height }}
      fallbackTable={
        <table className='table-xs table w-full'>
          <thead>
            <tr>
              <th className='text-left'>Class</th>
              <th className='text-right'>Precision</th>
              <th className='text-right'>Recall</th>
              <th className='text-right'>F1</th>
              <th className='text-right'>Support</th>
            </tr>
          </thead>
          <tbody>
            {data.map((d) => (
              <tr key={d.class}>
                <td>{d.class}</td>
                <td className='text-right font-mono'>{d.precision.toFixed(3)}</td>
                <td className='text-right font-mono'>{d.recall.toFixed(3)}</td>
                <td className='text-right font-mono'>{d.f1.toFixed(3)}</td>
                <td className='text-right'>{d.support}</td>
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
          tickFormat={(v) => v.valueOf().toFixed(1)}
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
        {classes.map((cls) => {
          const classData = data.find((d) => d.class === cls);
          if (!classData) return null;
          return (
            <g key={cls} transform={`translate(${xScale(cls)!}, 0)`}>
              {METRICS.map((m) => (
                <Bar
                  key={m.key}
                  x={xScaleMetric(m.key)!}
                  y={yScale(classData[m.key as keyof typeof classData] as number)}
                  width={xScaleMetric.bandwidth()}
                  height={
                    innerHeight - yScale(classData[m.key as keyof typeof classData] as number)
                  }
                  rx={1.5}
                  ry={1.5}
                  className={cn(m.fill, 'transition-opacity hover:opacity-80')}
                />
              ))}
            </g>
          );
        })}
        {/* Legend */}
        <g transform={`translate(${innerWidth}, -10)`}>
          {METRICS.map((m, i) => (
            <g key={m.key} transform={`translate(${i * 80}, 0)`}>
              <rect x={0} y={0} width={12} height={12} rx={2} className={m.fill} />
              <text x={16} y={9} className='text-text-secondary text-xs'>
                {m.label}
              </text>
            </g>
          ))}
        </g>
        <text
          x={innerWidth / 2}
          y={innerHeight + 45}
          textAnchor='middle'
          className='text-text-tertiary text-xs'>
          Class
        </text>
        <text
          x={-innerHeight / 2}
          y={-40}
          transform='rotate(-90)'
          textAnchor='middle'
          className='text-text-tertiary text-xs'>
          Score
        </text>
      </g>
    </ChartWrapper>
  );
};
