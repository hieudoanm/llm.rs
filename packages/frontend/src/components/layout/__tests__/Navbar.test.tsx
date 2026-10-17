import type { ReactNode } from 'react';
import { render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { Navbar } from '../Navbar';

vi.mock('next/image', () => ({
  default: ({ alt = '', className }: { alt?: string; className?: string }) => {
    // eslint-disable-next-line @next/next/no-img-element -- stand-in for next/image in jsdom
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

const toggle = (): HTMLElement => screen.getByRole('button', { name: 'Toggle theme' });

describe('Navbar', () => {
  beforeEach(() => {
    document.documentElement.setAttribute('data-theme', 'skl-light');
    window.localStorage.clear();
  });

  it('links to the primary routes', () => {
    render(<Navbar />);

    const nav = screen.getByRole('navigation', { name: 'Primary' });
    const links = within(nav).getAllByRole('link');
    expect(links.map((link) => [link.getAttribute('href'), link.textContent])).toEqual([
      ['/jobs', 'Jobs'],
      ['/algorithms', 'Algorithms'],
    ]);
    expect(screen.getByRole('link', { name: 'skl' })).toHaveAttribute('href', '/');
  });

  it('renders the mobile menu with the same links', () => {
    render(<Navbar />);

    expect(screen.getByRole('button', { name: 'Open navigation' })).toBeInTheDocument();
    expect(screen.getAllByRole('link', { name: 'Jobs' })).toHaveLength(2);
  });

  it('toggles the theme between light and dark', async () => {
    render(<Navbar />);

    await userEvent.click(toggle());
    expect(document.documentElement).toHaveAttribute('data-theme', 'skl-dark');
    expect(window.localStorage.getItem('skl-theme')).toBe('skl-dark');

    await userEvent.click(toggle());
    expect(document.documentElement).toHaveAttribute('data-theme', 'skl-light');
    expect(window.localStorage.getItem('skl-theme')).toBe('skl-light');
  });

  it('still applies the theme when localStorage is unavailable', async () => {
    const setItem = vi.spyOn(Storage.prototype, 'setItem').mockImplementation(() => {
      throw new Error('storage blocked');
    });
    const consoleError = vi.spyOn(console, 'error').mockImplementation(() => undefined);

    render(<Navbar />);
    await userEvent.click(toggle());

    expect(document.documentElement).toHaveAttribute('data-theme', 'skl-dark');
    expect(consoleError).toHaveBeenCalledTimes(1);

    setItem.mockRestore();
    consoleError.mockRestore();
  });
});
