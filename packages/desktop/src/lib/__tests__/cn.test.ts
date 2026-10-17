import { describe, expect, it } from 'vitest';
import { cn } from '../cn';

describe('cn', () => {
  it('joins class names with a space', () => {
    expect(cn('a', 'b')).toBe('a b');
  });

  it('drops falsy values', () => {
    expect(cn('a', undefined, null, false, '', 'b')).toBe('a b');
  });

  it('supports arrays and conditional objects', () => {
    expect(cn(['a', { b: true, c: false }])).toBe('a b');
  });

  it('resolves conflicting Tailwind utilities, last one wins', () => {
    expect(cn('p-2', 'p-4')).toBe('p-4');
    expect(cn('text-sm', 'text-lg')).toBe('text-lg');
  });

  it('keeps classes that do not conflict', () => {
    expect(cn('flex items-center', 'text-sm')).toBe('flex items-center text-sm');
  });
});
