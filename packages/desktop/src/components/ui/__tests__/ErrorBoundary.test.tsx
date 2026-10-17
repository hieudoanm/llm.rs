import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import type { ReactNode } from 'react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { ErrorBoundary } from '../ErrorBoundary';

/** A child that throws on render while `shouldThrow` is true. */
const Boom = ({ shouldThrow }: { shouldThrow: boolean }): ReactNode => {
  if (shouldThrow) throw new Error('The chart component exploded');
  return <p>All good</p>;
};

describe('ErrorBoundary', () => {
  beforeEach(() => {
    // React logs the caught error; keep the test output clean.
    vi.spyOn(console, 'error').mockImplementation(() => {});
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('renders its children when nothing throws', () => {
    render(
      <ErrorBoundary>
        <Boom shouldThrow={false} />
      </ErrorBoundary>,
    );

    expect(screen.getByText('All good')).toBeInTheDocument();
    expect(screen.queryByRole('alert')).not.toBeInTheDocument();
  });

  it('shows the studio alert with the error message when a child throws', () => {
    render(
      <ErrorBoundary>
        <Boom shouldThrow />
      </ErrorBoundary>,
    );

    const alert = screen.getByRole('alert');
    expect(alert).toHaveTextContent('Something went wrong');
    expect(alert).toHaveTextContent('The chart component exploded');
  });

  it('uses the caller title and guidance', () => {
    render(
      <ErrorBoundary
        title='This job view could not be displayed'
        guidance='Return to the jobs list.'>
        <Boom shouldThrow />
      </ErrorBoundary>,
    );

    const alert = screen.getByRole('alert');
    expect(alert).toHaveTextContent('This job view could not be displayed');
    expect(alert).toHaveTextContent('Return to the jobs list.');
  });

  it('recovers the subtree when Try again is pressed', async () => {
    const { rerender } = render(
      <ErrorBoundary>
        <Boom shouldThrow />
      </ErrorBoundary>,
    );

    expect(screen.getByRole('alert')).toBeInTheDocument();

    // The cause is fixed before the reset, so the re-render succeeds.
    rerender(
      <ErrorBoundary>
        <Boom shouldThrow={false} />
      </ErrorBoundary>,
    );
    await userEvent.click(screen.getByRole('button', { name: 'Try again' }));

    expect(screen.getByText('All good')).toBeInTheDocument();
    expect(screen.queryByRole('alert')).not.toBeInTheDocument();
  });

  it('renders a custom fallback when one is supplied', () => {
    render(
      <ErrorBoundary
        fallback={(error, reset) => (
          <button type='button' onClick={reset}>
            {`Recover from: ${error.message}`}
          </button>
        )}>
        <Boom shouldThrow />
      </ErrorBoundary>,
    );

    expect(
      screen.getByRole('button', { name: 'Recover from: The chart component exploded' }),
    ).toBeInTheDocument();
  });

  it('falls back to a generic message when the error has none', () => {
    const Silent = (): ReactNode => {
      throw new Error('');
    };

    render(
      <ErrorBoundary>
        <Silent />
      </ErrorBoundary>,
    );

    expect(screen.getByRole('alert')).toHaveTextContent(
      'An unexpected error occurred while rendering this view.',
    );
  });
});
