import { render, screen } from '@testing-library/react';
import Bands from '../Bands';

describe('Bands', () => {
  it('renders the band text', () => {
    render(<Bands />);
    expect(screen.getByText(/train on their own machine/i)).toBeInTheDocument();
  });

  it('renders category labels', () => {
    render(<Bands />);
    expect(screen.getByText(/data/i)).toBeInTheDocument();
    expect(screen.getByText(/models/i)).toBeInTheDocument();
    expect(screen.getByText(/metrics/i)).toBeInTheDocument();
    expect(screen.getByText(/onnx/i)).toBeInTheDocument();
  });
});
