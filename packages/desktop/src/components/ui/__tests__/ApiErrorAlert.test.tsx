import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { ApiErrorAlert } from '../ApiErrorAlert';
import { ApiError } from '@/lib/api';

describe('ApiErrorAlert', () => {
  it('shows the catalogue guidance of the backend code', () => {
    const error = new ApiError(
      'HTTP_ERROR',
      'Column "price" is missing',
      422,
      'req-1',
      undefined,
      'INSUFFICIENT_LABELS',
    );

    render(<ApiErrorAlert error={error} />);

    const alert = screen.getByRole('alert');
    expect(alert).toHaveTextContent('Not enough labels');
    expect(alert).toHaveTextContent('Semi-supervised needs at least 10 labelled rows');
    expect(alert).toHaveTextContent('Request id: req-1');
  });

  it('lists the raw code and the status in the details', () => {
    const error = new ApiError('HTTP_ERROR', 'boom', 500, 'req-1', undefined, 'INTERNAL_ERROR');

    render(<ApiErrorAlert error={error} />);

    expect(screen.getByText('Backend code: INTERNAL_ERROR')).toBeInTheDocument();
    expect(screen.getByText('HTTP status: 500')).toBeInTheDocument();
  });

  it('offers a retry action when a handler is given', async () => {
    const onRetry = vi.fn();
    const error = new ApiError('NETWORK_ERROR', 'boom', 0);

    render(<ApiErrorAlert error={error} onRetry={onRetry} />);

    await userEvent.click(screen.getByRole('button', { name: 'Retry' }));
    expect(onRetry).toHaveBeenCalledTimes(1);
  });

  it('offers the catalogue action when there is no retry handler', () => {
    const error = new ApiError('HTTP_ERROR', 'boom', 404, undefined, undefined, 'JOB_NOT_FOUND');

    render(<ApiErrorAlert error={error} />);

    const link = screen.getByRole('link', { name: 'Back to jobs' });
    expect(link).toHaveAttribute('href', '/jobs');
    // The guidance is not retryable, so the action is a ghost button.
    expect(link).toHaveClass('btn-ghost');
  });

  it('prefers the catalogue remedy over a retry when retrying cannot help', () => {
    const onRetry = vi.fn();
    const error = new ApiError('HTTP_ERROR', 'boom', 404, undefined, undefined, 'JOB_NOT_FOUND');

    render(<ApiErrorAlert error={error} onRetry={onRetry} />);

    expect(screen.getByRole('link', { name: 'Back to jobs' })).toHaveAttribute('href', '/jobs');
    expect(screen.queryByRole('button', { name: 'Retry' })).not.toBeInTheDocument();
  });

  it('labels the retry action with the catalogue action', async () => {
    const onRetry = vi.fn();
    const error = new ApiError('HTTP_ERROR', 'boom', 500, undefined, undefined, 'EXPORT_FAILED');

    render(<ApiErrorAlert error={error} onRetry={onRetry} />);

    await userEvent.click(screen.getByRole('button', { name: 'Re-run job' }));
    expect(onRetry).toHaveBeenCalledTimes(1);
  });

  it('falls back to a plain message for a non-API error', () => {
    render(<ApiErrorAlert error={new Error('Something odd')} />);

    expect(screen.getByRole('alert')).toHaveTextContent('Something odd');
  });
});
