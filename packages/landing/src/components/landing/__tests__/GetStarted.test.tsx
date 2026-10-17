import { render, screen } from '@testing-library/react';
import GetStarted from '../GetStarted';

describe('GetStarted', () => {
  it('renders section heading', () => {
    render(<GetStarted />);
    expect(screen.getByText(/start with a csv/i)).toBeInTheDocument();
    expect(screen.getByText(/train on your machine/i)).toBeInTheDocument();
    expect(screen.getByText(/export everywhere/i)).toBeInTheDocument();
  });

  it('renders description', () => {
    render(<GetStarted />);
    expect(screen.getByText(/open source/i)).toBeInTheDocument();
    expect(screen.getByText(/rust-powered/i)).toBeInTheDocument();
    expect(screen.getByText(/portable onnx artefact/i)).toBeInTheDocument();
  });

  it('renders call-to-action links', () => {
    render(<GetStarted />);
    expect(screen.getByRole('link', { name: /view the repository/i })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: /read the docs/i })).toBeInTheDocument();
  });
});
