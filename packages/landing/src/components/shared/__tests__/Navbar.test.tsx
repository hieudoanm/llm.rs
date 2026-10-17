import { render, screen } from '@testing-library/react';
import Navbar from '../Navbar';

describe('Navbar', () => {
  it('renders llm.rs home link', () => {
    render(<Navbar />);
    expect(screen.getByRole('link', { name: /llm\.rs home/i })).toBeInTheDocument();
    expect(screen.getByText('llm.rs')).toBeInTheDocument();
  });

  it('renders primary navigation links', () => {
    render(<Navbar />);
    expect(screen.getByRole('link', { name: /why llm\.rs/i })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: /workflow/i })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: /architecture/i })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: /docs/i })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: /get started/i })).toBeInTheDocument();
  });

  it('renders GitHub link', () => {
    render(<Navbar />);
    expect(screen.getByRole('link', { name: /github/i })).toBeInTheDocument();
  });

  it('renders header with primary navigation aria-label', () => {
    render(<Navbar />);
    expect(screen.getByRole('navigation', { name: 'Primary navigation' })).toBeInTheDocument();
  });
});
