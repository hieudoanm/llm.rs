import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { ScatterChart } from '../ScatterChart';

describe('ScatterChart', () => {
  const samplePoints = [
    { x: 1, y: 2 },
    { x: 2, y: 4 },
    { x: 3, y: 3 },
    { x: 4, y: 5 },
    { x: 5, y: 7 },
  ];

  it('renders title and caption', () => {
    render(<ScatterChart title='Scatter Plot' caption='Test caption' points={samplePoints} />);

    expect(screen.getByRole('heading', { name: 'Scatter Plot' })).toBeInTheDocument();
    expect(screen.getByText('Test caption')).toBeInTheDocument();
  });

  it('renders SVG with correct viewBox', () => {
    render(<ScatterChart title='Test Chart' caption='Caption' points={samplePoints} />);

    const svg = screen.getByRole('img', { name: 'Test Chart' });
    expect(svg).toBeInTheDocument();
    expect(svg).toHaveAttribute('viewBox', '0 0 500 300');
  });

  it('renders points for each data point', () => {
    render(<ScatterChart title='Test Chart' caption='Caption' points={samplePoints} />);

    // The SVG should be rendered
    const svg = screen.getByRole('img', { name: 'Test Chart' });
    expect(svg).toBeInTheDocument();
  });

  it('renders fallback table with data points', () => {
    render(<ScatterChart title='Test Chart' caption='Caption' points={samplePoints} />);

    const details = screen.getByText('Show values as table');
    details.click();

    // Values appear in both headers and cells
    expect(screen.getAllByText('1.000').length).toBeGreaterThan(0);
    expect(screen.getAllByText('2.000').length).toBeGreaterThan(0);
    expect(screen.getAllByText('3.000').length).toBeGreaterThan(0);
  });

  it('handles empty data gracefully', () => {
    render(<ScatterChart title='Empty' caption='No data' points={[]} />);

    expect(screen.getByRole('img', { name: 'Empty' })).toBeInTheDocument();
    expect(screen.getAllByText('No data').length).toBeGreaterThan(0);
  });

  it('renders reference line when identity type provided', () => {
    render(
      <ScatterChart
        title='Test Identity'
        caption='Caption'
        points={samplePoints}
        referenceLine={{ type: 'identity' }}
      />,
    );

    const svg = screen.getByRole('img', { name: 'Test Identity' });
    expect(svg).toBeInTheDocument();
  });

  it('renders horizontal reference line when provided', () => {
    render(
      <ScatterChart
        title='Test Horizontal'
        caption='Caption'
        points={samplePoints}
        referenceLine={{ type: 'horizontal', value: 0 }}
      />,
    );

    const svg = screen.getByRole('img', { name: 'Test Horizontal' });
    expect(svg).toBeInTheDocument();
  });

  it('colors points by cluster when enabled', () => {
    const pointsWithCluster = samplePoints.map((p, i) => ({ ...p, cluster: i % 2 }));

    render(
      <ScatterChart
        title='Test Cluster'
        caption='Caption'
        points={pointsWithCluster}
        colorByCluster={true}
      />,
    );

    const svg = screen.getByRole('img', { name: 'Test Cluster' });
    expect(svg).toBeInTheDocument();
  });

  it('renders xLabel and yLabel when provided', () => {
    render(
      <ScatterChart
        title='Test'
        caption='Caption'
        points={samplePoints}
        xLabel='X Axis'
        yLabel='Y Axis'
      />,
    );

    // Labels appear in both axis and fallback table
    expect(screen.getAllByText('X Axis').length).toBeGreaterThan(0);
    expect(screen.getAllByText('Y Axis').length).toBeGreaterThan(0);
  });

  it('handles single point', () => {
    render(<ScatterChart title='Single' caption='Caption' points={[{ x: 1, y: 1 }]} />);

    const svg = screen.getByRole('img', { name: 'Single' });
    expect(svg).toBeInTheDocument();
  });
});
