import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { ChartWrapper } from '../ChartWrapper';

describe('ChartWrapper', () => {
  it('renders title and caption', () => {
    render(
      <ChartWrapper title='Test Chart' caption='Test caption'>
        <text x='100' y='100'>
          Chart content
        </text>
      </ChartWrapper>,
    );

    expect(screen.getByRole('heading', { name: 'Test Chart' })).toBeInTheDocument();
    expect(screen.getByText('Test caption')).toBeInTheDocument();
  });

  it('renders SVG with correct viewBox', () => {
    render(
      <ChartWrapper title='Test Chart' caption='Caption'>
        <text x='100' y='100'>
          Chart content
        </text>
      </ChartWrapper>,
    );

    const svg = screen.getByRole('img', { name: 'Test Chart' });
    expect(svg).toBeInTheDocument();
    expect(svg).toHaveAttribute('viewBox', '0 0 500 300');
  });

  it('renders custom viewBox when provided', () => {
    render(
      <ChartWrapper title='Test Chart' caption='Caption' viewBox={{ width: 800, height: 600 }}>
        <text x='100' y='100'>
          Chart content
        </text>
      </ChartWrapper>,
    );

    const svg = screen.getByRole('img', { name: 'Test Chart' });
    expect(svg).toHaveAttribute('viewBox', '0 0 800 600');
  });

  it('renders fallback table when provided', () => {
    render(
      <ChartWrapper
        title='Test Chart'
        caption='Caption'
        fallbackTable={
          <table>
            <tbody>
              <tr>
                <td>Cell</td>
              </tr>
            </tbody>
          </table>
        }>
        <text x='100' y='100'>
          Chart content
        </text>
      </ChartWrapper>,
    );

    const details = screen.getByText('Show values as table');
    details.click();

    expect(screen.getByText('Cell')).toBeInTheDocument();
  });

  it('applies custom className', () => {
    render(
      <ChartWrapper title='Test Chart' caption='Caption' className='custom-class'>
        <text x='100' y='100'>
          Chart content
        </text>
      </ChartWrapper>,
    );

    const svg = screen.getByRole('img', { name: 'Test Chart' });
    // SVG -> relative container -> card-body -> card
    const card = svg.parentElement?.parentElement?.parentElement;
    expect(card).toHaveClass('custom-class');
  });

  it('renders children inside SVG', () => {
    render(
      <ChartWrapper title='Test Chart' caption='Caption'>
        <text x='100' y='100' data-testid='custom-text'>
          Custom content
        </text>
      </ChartWrapper>,
    );

    expect(screen.getByTestId('custom-text')).toBeInTheDocument();
  });
});
