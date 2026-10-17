import { render, screen } from '@testing-library/react';
import BuiltDifferent from '../BuiltDifferent';

describe('BuiltDifferent', () => {
  it('renders section heading', () => {
    render(<BuiltDifferent />);
    expect(screen.getByText(/built differently, on purpose/i)).toBeInTheDocument();
    expect(screen.getByText(/not a cloud service/i)).toBeInTheDocument();
    expect(screen.getByText(/not a black box/i)).toBeInTheDocument();
  });

  it('renders architecture components', () => {
    render(<BuiltDifferent />);
    expect(screen.getByText(/next\.js \+ tauri ui/i)).toBeInTheDocument();
    expect(screen.getByText('Rust training core')).toBeInTheDocument();
    expect(screen.getByText(/onnx artefact/i)).toBeInTheDocument();
  });

  it('renders feature list items', () => {
    render(<BuiltDifferent />);
    expect(screen.getByText(/rust-first foundation/i)).toBeInTheDocument();
    expect(screen.getByText(/reproducible training/i)).toBeInTheDocument();
    expect(screen.getByText(/portable by default/i)).toBeInTheDocument();
  });

  it('renders reproducibility text', () => {
    render(<BuiltDifferent />);
    expect(screen.getByText(/clear boundaries/i)).toBeInTheDocument();
    expect(screen.getByText(/reproducible models/i)).toBeInTheDocument();
  });
});
