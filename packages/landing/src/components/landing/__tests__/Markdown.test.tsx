import { render, screen } from '@testing-library/react';
import Markdown from '../Markdown';

describe('Markdown', () => {
  it('renders content', () => {
    render(<Markdown content='Hello World' />);
    expect(screen.getByTestId('markdown')).toBeInTheDocument();
  });

  it('renders headings', () => {
    render(<Markdown content={'# Title\n## Sub'} />);
    expect(screen.getByTestId('markdown')).toBeInTheDocument();
  });

  it('renders code blocks', () => {
    render(<Markdown content={'```rust\nlet x = 1;\n```'} />);
    expect(screen.getByTestId('markdown')).toBeInTheDocument();
  });
});
