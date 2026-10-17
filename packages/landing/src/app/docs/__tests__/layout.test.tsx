import { render, screen } from '@testing-library/react';
import { fireEvent } from '@testing-library/react';
import { vi } from 'vitest';

const { getDocsMock } = vi.hoisted(() => ({ getDocsMock: vi.fn() }));

vi.mock('@/lib/docs', () => ({ getDocs: getDocsMock }));

import DocsLayout, { metadata } from '../layout';

describe('docs layout', () => {
  beforeEach(() => {
    getDocsMock.mockReturnValue([
      { slug: '01-getting-started', title: 'Getting Started', description: 'Install llm.rs' },
      { slug: '05-architecture', title: 'Architecture', description: 'Crates and packages' },
    ]);
  });

  it('renders the shared navigation, document links, and page content', () => {
    render(
      <DocsLayout>
        <h1>Guide content</h1>
      </DocsLayout>,
    );

    expect(screen.getByRole('link', { name: 'llm.rs home' })).toHaveAttribute('href', '/');
    expect(screen.getByRole('navigation', { name: 'Primary navigation' })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Getting Started' })).toHaveAttribute(
      'href',
      '/docs/01-getting-started',
    );
    expect(screen.getByRole('link', { name: 'Architecture' })).toHaveAttribute(
      'href',
      '/docs/05-architecture',
    );
    expect(screen.getByRole('link', { name: /repository docs/i })).toHaveAttribute(
      'href',
      'https://github.com/hieudoanm/llm.rs/tree/main/docs',
    );
    expect(screen.getByRole('heading', { name: 'Guide content' })).toBeInTheDocument();
    expect(getDocsMock).toHaveBeenCalledOnce();
  });

  it('renders a search input in the sidebar', () => {
    render(
      <DocsLayout>
        <h1>Guide content</h1>
      </DocsLayout>,
    );
    expect(screen.getByRole('textbox', { name: /search/i })).toBeInTheDocument();
  });

  it('filters docs by title when searching', () => {
    render(
      <DocsLayout>
        <h1>Guide content</h1>
      </DocsLayout>,
    );
    const searchInput = screen.getByRole('textbox', { name: /search/i });
    fireEvent.change(searchInput, { target: { value: 'architecture' } });
    expect(screen.getByRole('link', { name: 'Architecture' })).toBeInTheDocument();
    expect(screen.queryByRole('link', { name: 'Getting Started' })).toBeNull();
  });

  it('filters docs by description when searching', () => {
    render(
      <DocsLayout>
        <h1>Guide content</h1>
      </DocsLayout>,
    );
    const searchInput = screen.getByRole('textbox', { name: /search/i });
    fireEvent.change(searchInput, { target: { value: 'install llm' } });
    expect(screen.getByRole('link', { name: 'Getting Started' })).toBeInTheDocument();
    expect(screen.queryByRole('link', { name: 'Architecture' })).toBeNull();
  });

  it('shows all docs when search is cleared', () => {
    render(
      <DocsLayout>
        <h1>Guide content</h1>
      </DocsLayout>,
    );
    const searchInput = screen.getByRole('textbox', { name: /search/i });
    fireEvent.change(searchInput, { target: { value: 'architecture' } });
    expect(screen.queryByRole('link', { name: 'Getting Started' })).toBeNull();
    fireEvent.change(searchInput, { target: { value: '' } });
    expect(screen.getByRole('link', { name: 'Getting Started' })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Architecture' })).toBeInTheDocument();
  });

  it('exports documentation metadata', () => {
    expect(metadata).toMatchObject({
      title: 'Documentation',
      description: expect.stringContaining('Guides for training models'),
    });
  });
});
