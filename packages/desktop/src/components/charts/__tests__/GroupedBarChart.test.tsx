import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { GroupedBarChart } from '../GroupedBarChart';

describe('GroupedBarChart', () => {
  const sampleData = [
    { class: 'A', precision: 0.9, recall: 0.85, f1: 0.87, support: 50 },
    { class: 'B', precision: 0.75, recall: 0.8, f1: 0.77, support: 30 },
    { class: 'C', precision: 0.95, recall: 0.9, f1: 0.92, support: 20 },
  ];

  it('renders title and caption', () => {
    render(<GroupedBarChart title='Per-Class Metrics' caption='Test caption' data={sampleData} />);

    expect(screen.getByRole('heading', { name: 'Per-Class Metrics' })).toBeInTheDocument();
    expect(screen.getByText('Test caption')).toBeInTheDocument();
  });

  it('renders SVG with correct viewBox', () => {
    render(<GroupedBarChart title='Test Chart' caption='Caption' data={sampleData} />);

    const svg = screen.getByRole('img', { name: 'Test Chart' });
    expect(svg).toBeInTheDocument();
    expect(svg).toHaveAttribute('viewBox', '0 0 600 300');
  });

  it('renders fallback table with data', () => {
    render(<GroupedBarChart title='Test Chart' caption='Caption' data={sampleData} />);

    const details = screen.getByText('Show values as table');
    details.click();

    // Labels appear in both headers and cells
    expect(screen.getAllByText('A').length).toBeGreaterThan(0);
    expect(screen.getAllByText('B').length).toBeGreaterThan(0);
    expect(screen.getAllByText('C').length).toBeGreaterThan(0);
    expect(screen.getAllByText('0.900').length).toBeGreaterThan(0);
    expect(screen.getAllByText('50').length).toBeGreaterThan(0);
  });

  it('handles empty data gracefully', () => {
    render(<GroupedBarChart title='Empty' caption='No data' data={[]} />);

    expect(screen.getByRole('img', { name: 'Empty' })).toBeInTheDocument();
    expect(screen.getAllByText('No data').length).toBeGreaterThan(0);
  });

  it('respects custom yMax', () => {
    render(<GroupedBarChart title='Test Chart' caption='Caption' data={sampleData} yMax={2.0} />);

    const svg = screen.getByRole('img', { name: 'Test Chart' });
    expect(svg).toBeInTheDocument();
  });
});
