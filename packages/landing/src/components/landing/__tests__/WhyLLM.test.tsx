import { render, screen } from '@testing-library/react';
import WhyLLM from '../WhyLLM';

describe('WhyLLM', () => {
  it('renders section heading', () => {
    render(<WhyLLM />);
    expect(screen.getByText(/from raw csv/i)).toBeInTheDocument();
    expect(screen.getByText(/portable model/i)).toBeInTheDocument();
  });

  it('renders eyebrow and paragraph', () => {
    render(<WhyLLM />);
    expect(screen.getByText(/one workflow\. every learning mode/i)).toBeInTheDocument();
    expect(screen.getByText(/local-first studio/i)).toBeInTheDocument();
  });

  it('renders all feature cards', () => {
    render(<WhyLLM />);
    expect(screen.getByText(/six task types, one workflow/i)).toBeInTheDocument();
    expect(screen.getByText(/local-first and private/i)).toBeInTheDocument();
    expect(screen.getByText(/onnx is the contract/i)).toBeInTheDocument();
  });

  it('renders feature footnotes', () => {
    render(<WhyLLM />);
    expect(screen.getByText(/no cloud account required/i)).toBeInTheDocument();
    expect(screen.getByText(/portable by design/i)).toBeInTheDocument();
  });

  it('renders code preview for the large feature card', () => {
    render(<WhyLLM />);
    expect(screen.getByText(/train a classifier/i)).toBeInTheDocument();
    expect(screen.getByText(/model\.onnx exported/i)).toBeInTheDocument();
  });
});
