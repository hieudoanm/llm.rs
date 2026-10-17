import { render, screen } from '@testing-library/react';
import Footer from '../Footer';

describe('Footer', () => {
  it('renders llm.rs home link', () => {
    render(<Footer />);
    expect(screen.getByRole('link', { name: /llm\.rs/i })).toBeInTheDocument();
  });

  it('renders tagline', () => {
    render(<Footer />);
    expect(screen.getByText(/csv in/i)).toBeInTheDocument();
    expect(screen.getByText(/onnx out/i)).toBeInTheDocument();
    expect(screen.getByText(/all on your machine/i)).toBeInTheDocument();
  });

  it('renders license and build info', () => {
    render(<Footer />);
    expect(screen.getByText(/mit licensed/i)).toBeInTheDocument();
    expect(screen.getByText(/built with rust/i)).toBeInTheDocument();
  });
});
