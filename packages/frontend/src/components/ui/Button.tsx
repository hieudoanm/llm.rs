'use client';

import type { ButtonHTMLAttributes, ReactNode } from 'react';
import { cn } from '@/lib/cn';

type ButtonVariant =
  | 'primary'
  | 'secondary'
  | 'accent'
  | 'neutral'
  | 'outline'
  | 'ghost'
  | 'success'
  | 'warning'
  | 'error';

type ButtonSize = 'sm' | 'md' | 'lg';

const VARIANT_CLASS: Record<ButtonVariant, string> = {
  primary: 'btn-primary',
  secondary: 'btn-secondary',
  accent: 'btn-accent',
  neutral: 'btn-neutral',
  outline: 'btn-outline',
  ghost: 'btn-ghost',
  success: 'btn-success',
  warning: 'btn-warning',
  error: 'btn-error',
};

const SIZE_CLASS: Record<ButtonSize, string> = {
  sm: 'btn-sm',
  md: 'btn-md',
  lg: 'btn-lg',
};

export interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  size?: ButtonSize;
  isLoading?: boolean;
  children?: ReactNode;
}

export const Button = ({
  className,
  variant = 'primary',
  size = 'md',
  isLoading = false,
  children,
  disabled,
  type = 'button',
  ...props
}: ButtonProps) => (
  <button
    type={type}
    disabled={disabled || isLoading}
    aria-busy={isLoading || undefined}
    className={cn('btn', VARIANT_CLASS[variant], SIZE_CLASS[size], className)}
    {...props}>
    {isLoading && <span className='loading loading-spinner loading-sm' aria-hidden='true' />}
    {children}
  </button>
);
