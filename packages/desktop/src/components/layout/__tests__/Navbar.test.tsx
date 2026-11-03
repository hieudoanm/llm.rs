import type { ReactNode } from 'react';
import { render, screen, within } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { Navbar } from '../Navbar';

vi.mock('next/image', () => ({
  default: ({ alt = '', className }: { alt?: string; className?: string }) => {
    return <img alt={alt} className={className} />;
  },
}));

vi.mock('next/link', () => ({
  default: ({
    href,
    className,
    'aria-label': ariaLabel,
    children,
  }: {
    href: string;
    className?: string;
    'aria-label'?: string;
    children?: ReactNode;
  }) => (
    <a href={href} className={className} aria-label={ariaLabel}>
      {children}
    </a>
  ),
}));

describe('Navbar', () => {
  it('links to the primary routes', () => {
    render(<Navbar />);

    const nav = screen.getByRole('navigation', { name: 'Primary' });
    const links = within(nav).getAllByRole('link');
    expect(links.map((link) => [link.getAttribute('href'), link.textContent])).toEqual([
      ['/jobs', 'Jobs'],
      ['/algorithms', 'Algorithms'],
    ]);
    expect(screen.getByRole('link', { name: 'llm' })).toHaveAttribute('href', '/');
  });

  it('renders the mobile menu with the same links', () => {
    render(<Navbar />);

    expect(screen.getByRole('button', { name: 'Open navigation' })).toBeInTheDocument();
    expect(screen.getAllByRole('link', { name: 'Jobs' })).toHaveLength(2);
  });
});
