import { render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';
import { Footer } from '../Footer';

const originalDocsUrl = process.env.NEXT_PUBLIC_API_DOCS_URL;

afterEach(() => {
  if (originalDocsUrl === undefined) {
    delete process.env.NEXT_PUBLIC_API_DOCS_URL;
  } else {
    process.env.NEXT_PUBLIC_API_DOCS_URL = originalDocsUrl;
  }
});

describe('Footer', () => {
  it('renders the local studio disclaimer', () => {
    render(<Footer />);

    expect(screen.getByText('scikit-learn-client — local studio.')).toBeInTheDocument();
    expect(screen.getByText('Not a production AutoML service.')).toBeInTheDocument();
  });

  it('links to the interactive API reference', () => {
    render(<Footer />);

    const link = screen.getByRole('link', { name: 'API documentation' });
    expect(link).toHaveAttribute('href', 'http://127.0.0.1:8000/docs');
    expect(link).toHaveAttribute('target', '_blank');
  });

  it('prefers the configured documentation URL', () => {
    process.env.NEXT_PUBLIC_API_DOCS_URL = 'https://example.test/reference';

    render(<Footer />);

    expect(screen.getByRole('link', { name: 'API documentation' })).toHaveAttribute(
      'href',
      'https://example.test/reference',
    );
  });

  it('falls back to the API base when the documentation URL is empty', () => {
    // The compose file passes an empty string when the variable is unset.
    process.env.NEXT_PUBLIC_API_DOCS_URL = '';

    render(<Footer />);

    expect(screen.getByRole('link', { name: 'API documentation' })).toHaveAttribute(
      'href',
      'http://127.0.0.1:8000/docs',
    );
  });
});
