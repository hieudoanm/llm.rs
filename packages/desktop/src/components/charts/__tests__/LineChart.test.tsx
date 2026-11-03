import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { LineChart } from '../LineChart';

describe('LineChart', () => {
  const sampleData = [5, 10, 8, 12, 9, 14, 11];

  it('renders title and caption', () => {
    render(<LineChart title='Training Progress' caption='Test caption' data={sampleData} />);

    expect(screen.getByRole('heading', { name: 'Training Progress' })).toBeInTheDocument();
    expect(screen.getByText('Test caption')).toBeInTheDocument();
  });

  it('renders SVG with correct viewBox', () => {
    render(<LineChart title='Training Progress' caption='Caption' data={sampleData} />);

    const svg = screen.getByRole('img', { name: 'Training Progress' });
    expect(svg).toBeInTheDocument();
    expect(svg).toHaveAttribute('viewBox', '0 0 500 300');
  });

  it('renders fallback table with data points', () => {
    render(<LineChart title='Test Chart' caption='Caption' data={sampleData} />);

    const details = screen.getByText('Show values as table');
    details.click();

    // Use getAllByText and check counts since values appear in both headers and cells
    expect(screen.getAllByText('1').length).toBeGreaterThan(0);
    expect(screen.getAllByText('2').length).toBeGreaterThan(0);
    expect(screen.getAllByText('5').length).toBeGreaterThan(0);
    expect(screen.getAllByText('10').length).toBeGreaterThan(0);
  });

  it('handles empty data gracefully', () => {
    render(<LineChart title='Empty' caption='No data' data={[]} />);

    expect(screen.getByRole('img', { name: 'Empty' })).toBeInTheDocument();
    expect(screen.getAllByText('No data').length).toBeGreaterThan(0);
  });

  it('renders secondary data when provided', () => {
    render(
      <LineChart title='Test Chart' caption='Caption' data={[1, 2, 3]} secondaryData={[2, 4, 6]} />,
    );

    const svg = screen.getByRole('img', { name: 'Test Chart' });
    expect(svg).toBeInTheDocument();
    // Both lines should be rendered
    const paths = svg.querySelectorAll('path[stroke]');
    expect(paths.length).toBeGreaterThanOrEqual(2);
  });

  it('respects yZero prop', () => {
    render(<LineChart title='Test Chart' caption='Caption' data={[-5, -2, 0, 3]} yZero={false} />);

    const svg = screen.getByRole('img', { name: 'Test Chart' });
    expect(svg).toBeInTheDocument();
  });

  it('renders xLabel and yLabel when provided', () => {
    render(
      <LineChart
        title='Test Chart'
        caption='Caption'
        data={sampleData}
        xLabel='Iteration'
        yLabel='Newly Labelled'
      />,
    );

    // These labels appear in the axis and fallback table
    expect(screen.getAllByText('Iteration').length).toBeGreaterThan(0);
    expect(screen.getAllByText('Newly Labelled').length).toBeGreaterThan(0);
  });

  it('handles single data point', () => {
    render(<LineChart title='Single' caption='Caption' data={[42]} />);

    const svg = screen.getByRole('img', { name: 'Single' });
    expect(svg).toBeInTheDocument();
  });
});
