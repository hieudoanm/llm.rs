'use client';

import Image from 'next/image';
import Link from 'next/link';
import { PiList } from 'react-icons/pi';

const NAV_LINKS = [
  { href: '/jobs', label: 'Jobs' },
  { href: '/algorithms', label: 'Algorithms' },
] as const;

export const Navbar = () => (
  <header className='bg-background/80 border-border supports-[backdrop-filter]:bg-background/60 sticky top-0 z-40 border-b backdrop-blur'>
    <div className='navbar mx-auto min-h-16 max-w-5xl px-4'>
      <div className='navbar-start'>
        <Link href='/' aria-label='llm' className='flex items-center gap-2'>
          <Image
            src='/brand/logo-mark-dark.svg'
            alt='llm'
            aria-hidden='true'
            width={24}
            height={24}
            unoptimized
            className='h-6 w-6'
          />
        </Link>
      </div>
      <div className='navbar-center hidden md:flex'>
        <nav aria-label='Primary' className='flex items-center gap-6'>
          {NAV_LINKS.map(({ href, label }) => (
            <Link
              key={href}
              href={href}
              className='text-text-secondary hover:text-text-primary text-sm font-medium'>
              {label}
            </Link>
          ))}
        </nav>
      </div>
      <div className='navbar-end gap-1'>
        <div className='dropdown dropdown-end md:hidden'>
          <button
            type='button'
            tabIndex={0}
            aria-label='Open navigation'
            className='btn btn-ghost btn-sm px-2'>
            <PiList className='icon-md' aria-hidden='true' />
          </button>
          <ul
            tabIndex={0}
            className='dropdown-content menu bg-base-100 rounded-box z-50 mt-3 w-44 p-2 shadow'>
            {NAV_LINKS.map(({ href, label }) => (
              <li key={href}>
                <Link href={href}>{label}</Link>
              </li>
            ))}
          </ul>
        </div>
      </div>
    </div>
  </header>
);
