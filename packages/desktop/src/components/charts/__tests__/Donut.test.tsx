import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { Donut } from '../Donut';

describe('Donut', () => {
  const sampleData = [
    { label: 'A', value: 30 },
    { label: 'B', value: 20 },
    { label: 'C', value: 50 },
  ];

  it('renders title and caption', () => {
    render(<Donut title='Distribution' caption='Test caption' data={sampleData} />);

    expect(screen.getByRole('heading', { name: 'Distribution' })).toBeInTheDocument();
    expect(screen.getByText('Test caption')).toBeInTheDocument();
  });

  it('renders SVG with correct viewBox', () => {
    render(<Donut title='Test Chart' caption='Caption' data={sampleData} />);

    const svg = screen.getByRole('img', { name: 'Test Chart' });
    expect(svg).toBeInTheDocument();
    expect(svg).toHaveAttribute('viewBox', '0 0 250 250');
  });

  it('renders fallback table with data', () => {
    render(<Donut title='Test Chart' caption='Caption' data={sampleData} />);

    const details = screen.getByText('Show values as table');
    details.click();

    // Values appear in both center label and table cells
    expect(screen.getAllByText('A').length).toBeGreaterThan(0);
    expect(screen.getAllByText('B').length).toBeGreaterThan(0);
    expect(screen.getAllByText('C').length).toBeGreaterThan(0);
    expect(screen.getAllByText('30').length).toBeGreaterThan(0);
    expect(screen.getAllByText('20').length).toBeGreaterThan(0);
    expect(screen.getAllByText('50').length).toBeGreaterThan(0);
    expect(screen.getAllByText('100').length).toBeGreaterThan(0);
  });

  it('handles empty data gracefully', () => {
    render(<Donut title='Empty' caption='No data' data={[]} />);

    expect(screen.getByRole('img', { name: 'Empty' })).toBeInTheDocument();
    expect(screen.getAllByText('No data').length).toBeGreaterThan(0);
  });

  it('handles zero values gracefully', () => {
    render(
      <Donut
        title='Zero'
        caption='All zero'
        data={[
          { label: 'A', value: 0 },
          { label: 'B', value: 0 },
        ]}
      />,
    );

    expect(screen.getByRole('img', { name: 'Zero' })).toBeInTheDocument();
    expect(screen.getAllByText('No data').length).toBeGreaterThan(0);
  });

  it('uses total prop when provided', () => {
    render(
      <Donut title='Test Chart' caption='Caption' data={[{ label: 'A', value: 10 }]} total={100} />,
    );

    const details = screen.getByText('Show values as table');
    details.click();

    // 100 appears in both center label and table
    expect(screen.getAllByText('10').length).toBeGreaterThan(0);
    expect(screen.getAllByText('100').length).toBeGreaterThan(0);
  });

  it('renders center total label', () => {
    render(<Donut title='Test Chart' caption='Caption' data={sampleData} />);

    // 100 appears in both center label and table
    expect(screen.getAllByText('100').length).toBeGreaterThan(0);
  });
});
