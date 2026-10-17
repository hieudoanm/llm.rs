'use client';

import { pie, arc } from '@visx/shape';
import { scaleOrdinal } from '@visx/scale';
import { ChartWrapper } from './ChartWrapper';

interface DonutSegment {
  label: string;
  value: number;
  fill?: string;
}

interface DonutProps {
  title: string;
  caption: string;
  data: DonutSegment[];
  /** Total value for percentage display. */
  total?: number;
}

const DEFAULT_COLORS = [
  'var(--color-primary)',
  'var(--color-secondary)',
  'var(--color-accent)',
  'var(--color-info)',
  'var(--color-success)',
  'var(--color-warning)',
  'var(--color-error)',
];

export const Donut = ({ title, caption, data, total }: DonutProps) => {
  if (!data.length || data.every((d) => d.value === 0)) {
    return (
      <ChartWrapper title={title} caption={caption}>
        <text
          x={150}
          y={150}
          textAnchor='middle'
          dominantBaseline='middle'
          className='text-text-tertiary'>
          No data
        </text>
      </ChartWrapper>
    );
  }

  const size = 250;
  const radius = size / 2;
  const innerRadius = radius * 0.6;
  const center = { x: radius, y: radius };
  const sum = total ?? data.reduce((a, b) => a + b.value, 0);

  const colorScale = scaleOrdinal<string>()
    .domain(data.map((d) => d.label))
    .range(DEFAULT_COLORS);
  const pieGen = pie<DonutSegment>()
    .value((d) => d.value)
    .sort(null);
  const arcs = pieGen(data);
  const arcGen = arc<DonutSegment>()
    .innerRadius(innerRadius)
    .outerRadius(radius)
    .padAngle(0.02)
    .padRadius(radius * 1.5);

  return (
    <ChartWrapper
      title={title}
      caption={caption}
      viewBox={{ width: size, height: size }}
      fallbackTable={
        <table className='table-xs mx-auto table w-full max-w-xs'>
          <thead>
            <tr>
              <th className='text-left'>Label</th>
              <th className='text-right'>Count</th>
              <th className='text-right'>%</th>
            </tr>
          </thead>
          <tbody>
            {data.map((d) => (
              <tr key={d.label}>
                <td>{d.label}</td>
                <td className='text-right'>{d.value}</td>
                <td className='text-right font-mono'>
                  {sum > 0 ? ((d.value / sum) * 100).toFixed(1) : '0.0'}%
                </td>
              </tr>
            ))}
            <tr>
              <td className='font-medium'>Total</td>
              <td className='text-right font-medium'>{sum}</td>
              <td className='text-right font-mono font-medium'>100.0%</td>
            </tr>
          </tbody>
        </table>
      }>
      <g transform={`translate(${center.x}, ${center.y})`}>
        {arcs.map((a, i) => (
          <path
            key={i}
            // eslint-disable-next-line @typescript-eslint/no-explicit-any
            d={arcGen(a as any)!}
            fill={(a.data.fill || String(colorScale(a.data.label)))!}
            stroke='var(--color-base-100)'
            strokeWidth={1}
            className='transition-opacity hover:opacity-80'
          />
        ))}
        {/* Center label */}
        <text
          x={0}
          y={0}
          textAnchor='middle'
          dominantBaseline='middle'
          className='text-text-primary text-lg font-bold'>
          {sum}
        </text>
        <text
          x={0}
          y={20}
          textAnchor='middle'
          dominantBaseline='middle'
          className='text-text-secondary text-xs'>
          total
        </text>
      </g>
    </ChartWrapper>
  );
};
