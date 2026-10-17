import { render, screen } from '@testing-library/react';
import Hero from '../Hero';

describe('Hero', () => {
  it('renders hero heading', () => {
    render(<Hero />);
    expect(screen.getByText(/csv in/i)).toBeInTheDocument();
    expect(screen.getByText(/onnx out/i)).toBeInTheDocument();
  });

  it('renders eyebrow and description', () => {
    render(<Hero />);
    expect(screen.getByText(/machine learning in rust/i)).toBeInTheDocument();
    expect(screen.getByText(/tabular data/i)).toBeInTheDocument();
  });

  it('renders primary and quiet buttons', () => {
    render(<Hero />);
    expect(screen.getByRole('link', { name: /train your first model/i })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: /view on github/i })).toBeInTheDocument();
  });

  it('renders feature highlights', () => {
    render(<Hero />);
    expect(screen.getByText(/six task types/i)).toBeInTheDocument();
    expect(screen.getByText(/onnx export/i)).toBeInTheDocument();
    expect(screen.getByText(/local-first by default/i)).toBeInTheDocument();
  });

  it('renders training preview with logo', () => {
    render(<Hero />);
    expect(screen.getByRole('img', { name: /llm\.rs logo/i })).toBeInTheDocument();
    expect(screen.getByText(/model\.onnx/i)).toBeInTheDocument();
  });
});
