import { render, screen } from '@testing-library/react';
import Page from '../page';

describe('Page', () => {
  it('renders', () => {
    render(<Page />);
    expect(screen.getAllByText(/llm\.rs|Docs/i).length).toBeGreaterThan(0);
  });
});
