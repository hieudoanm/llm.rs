import { render, screen } from '@testing-library/react';
import Workflow from '../Workflow';

describe('Workflow', () => {
  it('renders section heading', () => {
    render(<Workflow />);
    expect(screen.getByText(/from upload to artefact/i)).toBeInTheDocument();
    expect(screen.getByText(/a workflow that ends/i)).toBeInTheDocument();
    expect(screen.getByText(/with a portable model/i)).toBeInTheDocument();
  });

  it('renders all workflow steps', () => {
    render(<Workflow />);
    expect(screen.getByText('Upload')).toBeInTheDocument();
    expect(screen.getByText('Map')).toBeInTheDocument();
    expect(screen.getByText('Train')).toBeInTheDocument();
    expect(screen.getByText('Export')).toBeInTheDocument();
  });

  it('renders step numbers', () => {
    render(<Workflow />);
    expect(screen.getByText(/01/)).toBeInTheDocument();
    expect(screen.getByText(/02/)).toBeInTheDocument();
    expect(screen.getByText(/03/)).toBeInTheDocument();
    expect(screen.getByText(/04/)).toBeInTheDocument();
  });

  it('renders step descriptions', () => {
    render(<Workflow />);
    expect(screen.getByText(/register a csv with a header row/i)).toBeInTheDocument();
    expect(
      screen.getByText(/choose features x and, when required, a label y/i),
    ).toBeInTheDocument();
    expect(screen.getByText(/pick a task and algorithm, then fit the model/i)).toBeInTheDocument();
    expect(screen.getByText(/download the onnx artefact and its metrics/i)).toBeInTheDocument();
  });
});
