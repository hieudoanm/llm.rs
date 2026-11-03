import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { SilhouettePlot } from '../SilhouettePlot';

describe('SilhouettePlot', () => {
  const sampleSamples = [
    { value: 0.8, cluster: 0 },
    { value: 0.6, cluster: 0 },
    { value: 0.4, cluster: 1 },
    { value: 0.2, cluster: 1 },
  ];

  it('renders title and caption', () => {
    render(
      <SilhouettePlot
        title='Silhouette Plot'
        caption='Test caption'
        samples={sampleSamples}
        nClusters={2}
      />,
    );

    expect(screen.getByRole('heading', { name: 'Silhouette Plot' })).toBeInTheDocument();
    expect(screen.getByText('Test caption')).toBeInTheDocument();
  });

  it('renders SVG with correct viewBox', () => {
    render(
      <SilhouettePlot
        title='Silhouette Plot'
        caption='Caption'
        samples={sampleSamples}
        nClusters={2}
      />,
    );

    const svg = screen.getByRole('img', { name: 'Silhouette Plot' });
    expect(svg).toBeInTheDocument();
    expect(svg).toHaveAttribute('viewBox', '0 0 600 300');
  });

  it('renders fallback table with cluster data', () => {
    render(
      <SilhouettePlot title='Test Chart' caption='Caption' samples={sampleSamples} nClusters={2} />,
    );

    const details = screen.getByText('Show values as table');
    details.click();

    expect(screen.getByText('Cluster')).toBeInTheDocument();
    expect(screen.getByText('Mean Silhouette')).toBeInTheDocument();
    expect(screen.getByText('Size')).toBeInTheDocument();
    expect(screen.getByText('0')).toBeInTheDocument();
    expect(screen.getByText('1')).toBeInTheDocument();
    expect(screen.getByText('Overall')).toBeInTheDocument();
  });

  it('handles empty samples gracefully', () => {
    render(<SilhouettePlot title='Empty' caption='No data' samples={[]} nClusters={2} />);

    expect(screen.getByRole('img', { name: 'Empty' })).toBeInTheDocument();
    expect(screen.getByText('Silhouette undefined (all rows identical)')).toBeInTheDocument();
  });

  it('renders mean line label', () => {
    render(
      <SilhouettePlot title='Test Chart' caption='Caption' samples={sampleSamples} nClusters={2} />,
    );

    expect(screen.getByText('mean: 0.500')).toBeInTheDocument();
  });

  it('renders axis label', () => {
    render(
      <SilhouettePlot title='Test Chart' caption='Caption' samples={sampleSamples} nClusters={2} />,
    );

    expect(screen.getByText('Silhouette value')).toBeInTheDocument();
  });

  it('handles single cluster', () => {
    const singleClusterSamples = [
      { value: 0.5, cluster: 0 },
      { value: 0.7, cluster: 0 },
    ];
    render(
      <SilhouettePlot
        title='Single'
        caption='Caption'
        samples={singleClusterSamples}
        nClusters={1}
      />,
    );

    const svg = screen.getByRole('img', { name: 'Single' });
    expect(svg).toBeInTheDocument();
  });
});
