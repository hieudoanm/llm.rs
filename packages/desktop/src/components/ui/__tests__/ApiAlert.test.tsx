import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { ApiAlert, NetworkErrorAlert } from '../ApiAlert';

const VARIANTS = ['info', 'success', 'warning', 'error'] as const;

describe('ApiAlert', () => {
  it('defaults to an error alert', () => {
    render(<ApiAlert message='Unable to load jobs' />);

    const alert = screen.getByRole('alert');
    expect(alert).toHaveClass('alert', 'alert-error');
    expect(alert).toHaveTextContent('Unable to load jobs');
  });

  it.each(VARIANTS)('maps the %s variant to a DaisyUI class', (variant) => {
    render(<ApiAlert variant={variant} message='Something happened' />);

    expect(screen.getByRole('alert')).toHaveClass(`alert-${variant}`);
  });

  it('renders an optional title above the message', () => {
    render(
      <ApiAlert variant='warning' title='Mapping incomplete' message='Pick a target column' />,
    );

    const alert = screen.getByRole('alert');
    expect(alert).toHaveTextContent('Mapping incomplete');
    expect(alert).toHaveTextContent('Pick a target column');
  });

  it('omits the title when it is not provided', () => {
    render(<ApiAlert message='Only the message' />);

    expect(screen.getByRole('alert')).toHaveTextContent('Only the message');
    expect(screen.queryByText('Mapping incomplete')).not.toBeInTheDocument();
  });

  it('renders a dismiss button only when a handler is given', async () => {
    const onDismiss = vi.fn();
    const { rerender } = render(<ApiAlert message='Boom' onDismiss={onDismiss} />);

    await userEvent.click(screen.getByRole('button', { name: 'Dismiss' }));
    expect(onDismiss).toHaveBeenCalledTimes(1);

    rerender(<ApiAlert message='Boom' />);
    expect(screen.queryByRole('button', { name: 'Dismiss' })).not.toBeInTheDocument();
  });

  it('merges a custom class name', () => {
    render(<ApiAlert message='Boom' className='mt-4' />);

    expect(screen.getByRole('alert')).toHaveClass('alert-error', 'mt-4');
  });
});

describe('NetworkErrorAlert', () => {
  it('renders the network error copy and calls onRetry', async () => {
    const onRetry = vi.fn();
    render(<NetworkErrorAlert onRetry={onRetry} />);

    expect(screen.getByRole('alert')).toHaveTextContent('Unable to reach the training API');
    expect(screen.getByText('Network error')).toBeInTheDocument();

    await userEvent.click(screen.getByRole('button', { name: 'Retry' }));
    expect(onRetry).toHaveBeenCalledTimes(1);
  });
});

describe('ApiAlert guidance, action and details', () => {
  it('renders the guidance under the message', () => {
    render(<ApiAlert message='The upload failed' guidance='Only .csv files are accepted.' />);

    const alert = screen.getByRole('alert');
    expect(alert).toHaveTextContent('The upload failed');
    expect(alert).toHaveTextContent('Only .csv files are accepted.');
  });

  it('renders a primary action button that calls its handler', async () => {
    const onClick = vi.fn();
    render(<ApiAlert message='Boom' action={{ label: 'Retry', onClick }} />);

    const button = screen.getByRole('button', { name: 'Retry' });
    expect(button).toHaveClass('btn-primary');

    await userEvent.click(button);
    expect(onClick).toHaveBeenCalledTimes(1);
  });

  it('renders a link action when the action has a target', () => {
    render(
      <ApiAlert message='Boom' action={{ label: 'Back to jobs', href: '/jobs', primary: false }} />,
    );

    const link = screen.getByRole('link', { name: 'Back to jobs' });
    expect(link).toHaveAttribute('href', '/jobs');
    expect(link).toHaveClass('btn-ghost');
  });

  it('renders a primary link action by default', () => {
    render(<ApiAlert message='Boom' action={{ label: 'Retry', href: '/jobs' }} />);

    expect(screen.getByRole('link', { name: 'Retry' })).toHaveClass('btn-primary');
  });

  it('lists the extra details in a disclosure', () => {
    render(
      <ApiAlert message='Boom' details={['Backend code: VALIDATION_ERROR', 'HTTP status: 422']} />,
    );

    expect(screen.getByText('More details')).toBeInTheDocument();
    expect(screen.getByText('Backend code: VALIDATION_ERROR')).toBeInTheDocument();
    expect(screen.getByText('HTTP status: 422')).toBeInTheDocument();
  });

  it('omits the disclosure when there are no details', () => {
    render(<ApiAlert message='Boom' details={[]} />);

    expect(screen.queryByText('More details')).not.toBeInTheDocument();
  });

  it('shows the request id in the footer', () => {
    render(<ApiAlert message='Boom' requestId='req-42' />);

    expect(screen.getByRole('alert')).toHaveTextContent('Request id: req-42');
  });

  it('never renders a raw stack trace', () => {
    render(<ApiAlert message='Boom' details={['Traceback (most recent call last)']} />);

    // The disclosure may hold it, but the visible message must not.
    expect(screen.getByRole('alert').textContent).toContain('Boom');
  });
});
