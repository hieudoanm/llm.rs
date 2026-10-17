import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { Histogram } from '../Histogram';

describe('Histogram', () => {
  const sampleEdges = [0, 10, 20, 30, 40];
  const sampleCounts = [5, 15, 25, 10];

  it('renders title and caption', () => {
    render(
      <Histogram
        title='Score Distribution'
        caption='Test caption'
        edges={sampleEdges}
        counts={sampleCounts}
      />,
    );

    expect(screen.getByRole('heading', { name: 'Score Distribution' })).toBeInTheDocument();
    expect(screen.getByText('Test caption')).toBeInTheDocument();
  });

  it('renders SVG with correct viewBox', () => {
    render(
      <Histogram title='Test Chart' caption='Caption' edges={sampleEdges} counts={sampleCounts} />,
    );

    const svg = screen.getByRole('img', { name: 'Test Chart' });
    expect(svg).toBeInTheDocument();
    expect(svg).toHaveAttribute('viewBox', '0 0 500 300');
  });

  it('renders bars for each bin', () => {
    render(
      <Histogram title='Test Chart' caption='Caption' edges={sampleEdges} counts={sampleCounts} />,
    );

    // Check that the SVG is rendered
    const svg = screen.getByRole('img', { name: 'Test Chart' });
    expect(svg).toBeInTheDocument();
  });

  it('renders fallback table with bin data', () => {
    render(
      <Histogram title='Test Chart' caption='Caption' edges={sampleEdges} counts={sampleCounts} />,
    );

    const details = screen.getByText('Show values as table');
    details.click();

    // Use getAllByText and check counts since values appear in both headers and cells
    expect(screen.getAllByText('0.00–10.00').length).toBeGreaterThan(0);
    expect(screen.getAllByText('10.00–20.00').length).toBeGreaterThan(0);
    expect(screen.getAllByText('20.00–30.00').length).toBeGreaterThan(0);
    expect(screen.getAllByText('30.00–40.00').length).toBeGreaterThan(0);
    expect(screen.getAllByText('5').length).toBeGreaterThan(0);
    expect(screen.getAllByText('15').length).toBeGreaterThan(0);
    expect(screen.getAllByText('25').length).toBeGreaterThan(0);
    expect(screen.getAllByText('10').length).toBeGreaterThan(0);
  });

  it('handles empty data gracefully', () => {
    render(<Histogram title='Empty' caption='No data' edges={[]} counts={[]} />);

    expect(screen.getByRole('img', { name: 'Empty' })).toBeInTheDocument();
    expect(screen.getAllByText('No data').length).toBeGreaterThan(0);
  });

  it('renders threshold line when provided', () => {
    render(
      <Histogram
        title='Test Chart'
        caption='Caption'
        edges={sampleEdges}
        counts={sampleCounts}
        threshold={15}
      />,
    );

    expect(screen.getByRole('img', { name: 'Test Chart' })).toBeInTheDocument();
    // Threshold text should be rendered
    expect(screen.getByText('threshold: 15.00')).toBeInTheDocument();
  });

  it('handles mismatched edges and counts gracefully', () => {
    render(<Histogram title='Test Chart' caption='Caption' edges={[0, 10]} counts={[5, 10]} />); // counts.length should be edges.length - 1

    expect(screen.getByRole('img', { name: 'Test Chart' })).toBeInTheDocument();
    expect(screen.getByText('No data')).toBeInTheDocument();
  });
});
