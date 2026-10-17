import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { Button } from '../Button';

const VARIANTS = [
  'primary',
  'secondary',
  'accent',
  'neutral',
  'outline',
  'ghost',
  'success',
  'warning',
  'error',
] as const;

const SIZES = ['sm', 'md', 'lg'] as const;

describe('Button', () => {
  it('renders children with the default variant, size and type', () => {
    render(<Button>Save</Button>);

    const button = screen.getByRole('button', { name: 'Save' });
    expect(button).toHaveClass('btn', 'btn-primary', 'btn-md');
    expect(button).toHaveAttribute('type', 'button');
  });

  it.each(VARIANTS)('maps the %s variant to a DaisyUI class', (variant) => {
    render(<Button variant={variant}>Go</Button>);

    expect(screen.getByRole('button')).toHaveClass(`btn-${variant}`);
  });

  it.each(SIZES)('maps the %s size to a DaisyUI class', (size) => {
    render(<Button size={size}>Go</Button>);

    expect(screen.getByRole('button')).toHaveClass(`btn-${size}`);
  });

  it('disables itself and shows a spinner while loading', () => {
    render(<Button isLoading>Save</Button>);

    const button = screen.getByRole('button', { name: 'Save' });
    expect(button).toBeDisabled();
    expect(button).toHaveAttribute('aria-busy', 'true');
    expect(button.querySelector('.loading.loading-spinner')).not.toBeNull();
  });

  it('stays enabled and not busy when idle', () => {
    render(<Button>Save</Button>);

    const button = screen.getByRole('button', { name: 'Save' });
    expect(button).toBeEnabled();
    expect(button).not.toHaveAttribute('aria-busy');
    expect(button.querySelector('.loading')).toBeNull();
  });

  it('honours the disabled prop', () => {
    render(<Button disabled>Save</Button>);

    expect(screen.getByRole('button', { name: 'Save' })).toBeDisabled();
  });

  it('accepts an explicit type', () => {
    render(<Button type='submit'>Save</Button>);

    expect(screen.getByRole('button', { name: 'Save' })).toHaveAttribute('type', 'submit');
  });

  it('forwards click handlers and extra attributes', async () => {
    const onClick = vi.fn();
    render(
      <Button onClick={onClick} data-testid='submit-job' aria-label='Submit job'>
        Go
      </Button>,
    );

    await userEvent.click(screen.getByTestId('submit-job'));

    expect(onClick).toHaveBeenCalledTimes(1);
    expect(screen.getByLabelText('Submit job')).toBeInTheDocument();
  });

  it('merges a custom class name after the variant classes', () => {
    render(<Button className='w-full'>Go</Button>);

    expect(screen.getByRole('button')).toHaveClass('btn-primary', 'w-full');
  });
});
