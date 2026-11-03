import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { Heatmap } from '../Heatmap';

describe('Heatmap', () => {
  const sampleClasses = ['A', 'B', 'C'];
  const sampleMatrix = [
    [10, 2, 1],
    [1, 15, 3],
    [0, 2, 12],
  ];

  it('renders title and caption', () => {
    render(
      <Heatmap
        title='Confusion Matrix'
        caption='Test caption'
        classes={sampleClasses}
        matrix={sampleMatrix}
      />,
    );

    expect(screen.getByRole('heading', { name: 'Confusion Matrix' })).toBeInTheDocument();
    expect(screen.getByText('Test caption')).toBeInTheDocument();
  });

  it('renders SVG with correct viewBox', () => {
    render(
      <Heatmap
        title='Test Chart'
        caption='Caption'
        classes={sampleClasses}
        matrix={sampleMatrix}
      />,
    );

    const svg = screen.getByRole('img', { name: 'Test Chart' });
    expect(svg).toBeInTheDocument();
    expect(svg).toHaveAttribute('viewBox');
  });

  it('renders fallback table with class labels', () => {
    render(
      <Heatmap
        title='Test Chart'
        caption='Caption'
        classes={sampleClasses}
        matrix={sampleMatrix}
      />,
    );

    const details = screen.getByText('Show values as table');
    details.click();

    // Use getAllByText and check counts since labels appear in headers and cells
    expect(screen.getAllByText('True \\ Predicted').length).toBeGreaterThan(0);
    expect(screen.getAllByText('A').length).toBeGreaterThan(0);
    expect(screen.getAllByText('B').length).toBeGreaterThan(0);
    expect(screen.getAllByText('C').length).toBeGreaterThan(0);
    expect(screen.getAllByText('10').length).toBeGreaterThan(0);
    expect(screen.getAllByText('15').length).toBeGreaterThan(0);
    expect(screen.getAllByText('12').length).toBeGreaterThan(0);
  });

  it('handles empty data gracefully', () => {
    render(<Heatmap title='Empty' caption='No data' classes={[]} matrix={[]} />);

    expect(screen.getByRole('img', { name: 'Empty' })).toBeInTheDocument();
    expect(screen.getAllByText('No data').length).toBeGreaterThan(0);
  });

  it('renders class labels on axes', () => {
    render(
      <Heatmap
        title='Test Chart'
        caption='Caption'
        classes={sampleClasses}
        matrix={sampleMatrix}
      />,
    );

    // Class labels appear on both axes and in the table
    expect(screen.getAllByText('A').length).toBeGreaterThan(0);
    expect(screen.getAllByText('B').length).toBeGreaterThan(0);
    expect(screen.getAllByText('C').length).toBeGreaterThan(0);
  });
});
