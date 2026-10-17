import { render, screen } from '@testing-library/react';
import Tools from '../Tools';

describe('Tools', () => {
  it('renders section heading', () => {
    render(<Tools />);
    expect(screen.getByText(/meet llm\.rs where you work/i)).toBeInTheDocument();
    expect(screen.getByText(/one engine/i)).toBeInTheDocument();
    expect(screen.getByText(/your choice of surface/i)).toBeInTheDocument();
  });

  it('renders all tool cards', () => {
    render(<Tools />);
    expect(screen.getByText(/desktop app/i)).toBeInTheDocument();
    expect(screen.getByText(/command line/i)).toBeInTheDocument();
    expect(screen.getByText(/onnx runtime/i)).toBeInTheDocument();
  });

  it('renders tool tags', () => {
    render(<Tools />);
    expect(screen.getByText(/canonical/i)).toBeInTheDocument();
    expect(screen.getByText(/headless/i)).toBeInTheDocument();
    expect(screen.getByText(/portable/i)).toBeInTheDocument();
  });

  it('renders tool links', () => {
    render(<Tools />);
    expect(screen.getByText(/explore downloads/i)).toBeInTheDocument();
    expect(screen.getByText(/read the cli guide/i)).toBeInTheDocument();
    expect(screen.getByText(/read the workflow/i)).toBeInTheDocument();
  });
});
