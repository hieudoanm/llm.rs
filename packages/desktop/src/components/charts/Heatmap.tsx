'use client';

import { scaleLinear } from '@visx/scale';
import { ChartWrapper } from './ChartWrapper';

interface HeatmapProps {
  title: string;
  caption: string;
  /** Class names (row/col labels). */
  classes: string[];
  /** Confusion matrix: rows = true, cols = predicted. */
  matrix: number[][];
}

export const Heatmap = ({ title, caption, classes, matrix }: HeatmapProps) => {
  if (!classes.length || !matrix.length) {
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

  const size = 400;
  const cellSize = size / Math.max(classes.length, 1);
  const labelMargin = 60;
  const width = size + labelMargin * 2;
  const height = size + labelMargin * 2;

  // Find max value for colour scaling
  let maxVal = 0;
  for (const row of matrix) {
    for (const v of row) {
      if (v > maxVal) maxVal = v;
    }
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const colorScale: any = scaleLinear<number>();
  colorScale.domain([0, maxVal]);
  colorScale.range(['var(--color-base-100)', 'var(--color-primary)']);
  colorScale.clamp(true);

  return (
    <ChartWrapper
      title={title}
      caption={caption}
      viewBox={{ width, height }}
      fallbackTable={
        <div className='overflow-x-auto'>
          <table className='table-xs table w-full'>
            <thead>
              <tr>
                <th className='text-left'>True \ Predicted</th>
                {classes.map((c) => (
                  <th key={c} className='text-center font-mono text-xs'>
                    {c}
                  </th>
                ))}
              </tr>
            </thead>
            <tbody>
              {matrix.map((row, i) => (
                <tr key={classes[i]}>
                  <th className='text-left font-mono text-xs'>{classes[i]}</th>
                  {row.map((v, j) => (
                    <td key={j} className='text-center font-mono'>
                      {v}
                    </td>
                  ))}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      }>
      <g transform={`translate(${labelMargin}, ${labelMargin})`}>
        {/* Y-axis labels (true labels) */}
        {classes.map((cls, i) => (
          <text
            key={cls}
            x={-8}
            y={i * cellSize + cellSize / 2}
            textAnchor='end'
            dominantBaseline='middle'
            className='text-text-secondary text-xs'>
            {cls}
          </text>
        ))}
        {/* X-axis labels (predicted labels) */}
        {classes.map((cls, j) => (
          <text
            key={cls}
            x={j * cellSize + cellSize / 2}
            y={-8}
            textAnchor='middle'
            className='text-text-secondary text-xs'>
            {cls}
          </text>
        ))}
        {/* Cells */}
        {matrix.map((row, i) =>
          row.map((value, j) => (
            <rect
              key={`${i}-${j}`}
              x={j * cellSize}
              y={i * cellSize}
              width={cellSize}
              height={cellSize}
              fill={colorScale(value)}
              stroke='var(--color-base-300)'
              strokeWidth={0.5}
            />
          )),
        )}
        {/* Values */}
        {matrix.map((row, i) =>
          row.map((value, j) => (
            <text
              key={`${i}-${j}`}
              x={j * cellSize + cellSize / 2}
              y={i * cellSize + cellSize / 2 + 4}
              textAnchor='middle'
              dominantBaseline='middle'
              className='font-mono text-xs'
              fill={
                value > maxVal / 2 ? 'var(--color-primary-content)' : 'var(--color-base-content)'
              }>
              {value}
            </text>
          )),
        )}
      </g>
    </ChartWrapper>
  );
};
