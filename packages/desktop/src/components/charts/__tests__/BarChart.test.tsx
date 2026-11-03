import { render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { BarChart } from '../BarChart';

describe('BarChart', () => {
  const sampleData = [
    { label: 'A', value: 10 },
    { label: 'B', value: 20 },
    { label: 'C', value: 15 },
  ];

  it('renders title and caption', () => {
    render(<BarChart title='Test Chart' caption='Test caption' data={sampleData} />);

    expect(screen.getByRole('heading', { name: 'Test Chart' })).toBeInTheDocument();
    expect(screen.getByText('Test caption')).toBeInTheDocument();
  });

  it('renders SVG with correct viewBox', () => {
    render(<BarChart title='Test Chart' caption='Caption' data={sampleData} />);

    const svg = screen.getByRole('img', { name: 'Test Chart' });
    expect(svg).toBeInTheDocument();
    expect(svg).toHaveAttribute('viewBox', '0 0 500 300');
  });

  it('renders bars for each data point', () => {
    render(<BarChart title='Test Chart' caption='Caption' data={sampleData} />);

    // The SVG should be rendered
    const svg = screen.getByRole('img', { name: 'Test Chart' });
    expect(svg).toBeInTheDocument();
  });

  it('renders fallback table with data', () => {
    render(<BarChart title='Test Chart' caption='Caption' data={sampleData} />);

    // Open the details to show the table
    const details = screen.getByText('Show values as table');
    details.click();

    // Labels appear in both header and data rows
    expect(screen.getAllByText('A').length).toBeGreaterThan(0);
    expect(screen.getAllByText('B').length).toBeGreaterThan(0);
    expect(screen.getAllByText('C').length).toBeGreaterThan(0);
    expect(screen.getAllByText('10.00').length).toBeGreaterThan(0);
    expect(screen.getAllByText('20.00').length).toBeGreaterThan(0);
    expect(screen.getAllByText('15.00').length).toBeGreaterThan(0);
  });

  it('handles empty data gracefully', () => {
    render(<BarChart title='Empty' caption='No data' data={[]} />);

    expect(screen.getByRole('img', { name: 'Empty' })).toBeInTheDocument();
    expect(screen.getAllByText('No data').length).toBeGreaterThan(0);
  });

  it('applies custom formatValue function', () => {
    const customFormat = vi.fn((v: number) => `${v} units`);
    render(
      <BarChart
        title='Test Chart'
        caption='Caption'
        data={[{ label: 'X', value: 42 }]}
        formatValue={customFormat}
      />,
    );

    const details = screen.getByText('Show values as table');
    details.click();

    expect(customFormat).toHaveBeenCalledWith(42);
    expect(screen.getByText('42 units')).toBeInTheDocument();
  });

  it('applies custom fillClass', () => {
    render(
      <BarChart title='Test Chart' caption='Caption' data={sampleData} fillClass='fill-success' />,
    );

    // The SVG should be rendered
    const svg = screen.getByRole('img', { name: 'Test Chart' });
    expect(svg).toBeInTheDocument();
  });

  it('renders xLabel and yLabel when provided', () => {
    render(
      <BarChart
        title='Test Chart'
        caption='Caption'
        data={sampleData}
        xLabel='Categories'
        yLabel='Values'
      />,
    );

    // Labels appear in both axis and fallback table
    expect(screen.getAllByText('Categories').length).toBeGreaterThan(0);
    expect(screen.getAllByText('Values').length).toBeGreaterThan(0);
  });
});
