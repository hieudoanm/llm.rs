import { render, screen, fireEvent } from '@testing-library/react';
import ErrorComponent from '../error';

describe('Error', () => {
  it('renders error message', () => {
    render(<ErrorComponent error={new Error('Something broke')} reset={vi.fn()} />);
    expect(screen.getByText(/something went wrong/i)).toBeInTheDocument();
    expect(screen.getByText('Something broke')).toBeInTheDocument();
  });

  it('calls reset when Try again is clicked', () => {
    const reset = vi.fn();
    render(<ErrorComponent error={new Error('Something broke')} reset={reset} />);
    fireEvent.click(screen.getByRole('button', { name: /try again/i }));
    expect(reset).toHaveBeenCalledTimes(1);
  });

  it('renders back to docs link', () => {
    render(<ErrorComponent error={new Error('Something broke')} reset={vi.fn()} />);
    expect(screen.getByRole('link', { name: /back to docs/i })).toHaveAttribute('href', '/docs');
  });
});
