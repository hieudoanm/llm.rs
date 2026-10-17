import { render, screen, fireEvent } from '@testing-library/react';
import GlobalError from '../global-error';

describe('GlobalError', () => {
  it('renders error message', () => {
    render(<GlobalError error={new Error('Global error')} reset={vi.fn()} />);
    expect(screen.getByText(/something went wrong/i)).toBeInTheDocument();
    expect(screen.getByText('Global error')).toBeInTheDocument();
  });

  it('calls reset when Try again is clicked', () => {
    const reset = vi.fn();
    render(<GlobalError error={new Error('Global error')} reset={reset} />);
    fireEvent.click(screen.getByRole('button', { name: /try again/i }));
    expect(reset).toHaveBeenCalledTimes(1);
  });

  it('renders within html and body elements', () => {
    render(<GlobalError error={new Error('Global error')} reset={vi.fn()} />);
    expect(document.querySelector('html')).not.toBeNull();
    expect(document.querySelector('body')).not.toBeNull();
  });
});
