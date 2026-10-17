import { render, screen, fireEvent } from '@testing-library/react';
import DocsSearch from '../DocsSearch';

const docs = [
  {
    slug: '01-getting-started',
    title: 'Getting Started',
    description: 'Install llm.rs and run the desktop app',
    content: '# Getting Started',
  },
  {
    slug: '02-cli',
    title: 'CLI',
    description: 'llm.rs command-line reference',
    content: '# CLI',
  },
  {
    slug: '03-workflow',
    title: 'Workflow',
    description: 'Upload, map, train, and export',
    content: '# Workflow',
  },
];

describe('DocsSearch', () => {
  it('renders all docs when no search query', () => {
    render(<DocsSearch docs={docs} />);
    expect(screen.getByRole('link', { name: 'Getting Started' })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'CLI' })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Workflow' })).toBeInTheDocument();
  });

  it('renders search input', () => {
    render(<DocsSearch docs={docs} />);
    expect(screen.getByRole('textbox', { name: /search docs/i })).toBeInTheDocument();
  });

  it('filters docs by title', () => {
    render(<DocsSearch docs={docs} />);
    const input = screen.getByRole('textbox', { name: /search docs/i });
    fireEvent.change(input, { target: { value: 'cli' } });
    expect(screen.getByRole('link', { name: 'CLI' })).toBeInTheDocument();
    expect(screen.queryByRole('link', { name: 'Getting Started' })).toBeNull();
    expect(screen.queryByRole('link', { name: 'Workflow' })).toBeNull();
  });

  it('filters docs by description', () => {
    render(<DocsSearch docs={docs} />);
    const input = screen.getByRole('textbox', { name: /search docs/i });
    fireEvent.change(input, { target: { value: 'install' } });
    expect(screen.getByRole('link', { name: 'Getting Started' })).toBeInTheDocument();
    expect(screen.queryByRole('link', { name: 'CLI' })).toBeNull();
    expect(screen.queryByRole('link', { name: 'Workflow' })).toBeNull();
  });

  it('shows all docs when search is cleared', () => {
    render(<DocsSearch docs={docs} />);
    const input = screen.getByRole('textbox', { name: /search docs/i });
    fireEvent.change(input, { target: { value: 'cli' } });
    expect(screen.queryByRole('link', { name: 'Getting Started' })).toBeNull();
    fireEvent.change(input, { target: { value: '' } });
    expect(screen.getByRole('link', { name: 'Getting Started' })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'CLI' })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Workflow' })).toBeInTheDocument();
  });

  it('renders overview link', () => {
    render(<DocsSearch docs={docs} />);
    expect(screen.getByRole('link', { name: /overview/i })).toHaveAttribute('href', '/docs');
  });

  it('renders repository docs link', () => {
    render(<DocsSearch docs={docs} />);
    expect(screen.getByRole('link', { name: /repository docs/i })).toHaveAttribute(
      'href',
      'https://github.com/hieudoanm/llm.rs/tree/main/docs',
    );
  });

  it('matches search case-insensitively', () => {
    render(<DocsSearch docs={docs} />);
    const input = screen.getByRole('textbox', { name: /search docs/i });
    fireEvent.change(input, { target: { value: 'CLI' } });
    expect(screen.getByRole('link', { name: 'CLI' })).toBeInTheDocument();
  });
});
