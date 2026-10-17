'use client';

import type { ElementType } from 'react';
import { cn } from '@/lib/cn';
import { PiInfo, PiWarningCircle, PiCheckCircle } from 'react-icons/pi';

type AlertVariant = 'info' | 'success' | 'warning' | 'error';

const VARIANT_ICON: Record<AlertVariant, ElementType> = {
  info: PiInfo,
  success: PiCheckCircle,
  warning: PiWarningCircle,
  error: PiWarningCircle,
};

const VARIANT_CLASS: Record<AlertVariant, string> = {
  info: 'alert-info',
  success: 'alert-success',
  warning: 'alert-warning',
  error: 'alert-error',
};

export interface AlertAction {
  /** The label of the button. */
  label: string;
  /** Where the button navigates. */
  href?: string;
  /** What the button does when it is pressed. */
  onClick?: () => void;
  /**
   * True when the action is the primary remedy — a retry, typically.
   * The button is `btn-primary` then and `btn-ghost` otherwise.
   */
  primary?: boolean;
}

export interface ApiAlertProps {
  variant?: AlertVariant;
  title?: string;
  message: string;
  /** The actionable explanation shown under the message. */
  guidance?: string;
  /** The primary action offered next to the message. */
  action?: AlertAction;
  /** Extra bullet points shown in a disclosure. */
  details?: string[];
  /** The correlation id, shown in the footer so it can be quoted. */
  requestId?: string;
  onDismiss?: () => void;
  className?: string;
}

export const ApiAlert = ({
  variant = 'error',
  title,
  message,
  guidance,
  action,
  details,
  requestId,
  onDismiss,
  className,
}: ApiAlertProps) => {
  const Icon = VARIANT_ICON[variant];
  return (
    <div className={cn('alert', VARIANT_CLASS[variant], className)} role='alert'>
      <Icon className='icon-sm shrink-0' aria-hidden='true' />
      <div className='flex flex-col gap-0.5'>
        {title && <span className='text-sm font-semibold'>{title}</span>}
        <span className='text-sm'>{message}</span>
        {guidance && <span className='text-text-secondary text-sm'>{guidance}</span>}
        {details && details.length > 0 && (
          <details className='mt-1 text-xs'>
            <summary className='cursor-pointer'>More details</summary>
            <ul className='mt-1 list-disc pl-4'>
              {details.map((line) => (
                <li key={line}>{line}</li>
              ))}
            </ul>
          </details>
        )}
        {requestId && (
          <span className='text-text-tertiary mt-1 font-mono text-xs'>Request id: {requestId}</span>
        )}
      </div>
      {action && (
        <div className='ml-auto flex shrink-0 items-center gap-2'>
          {action.href ? (
            <a
              href={action.href}
              className={cn('btn btn-sm', action.primary === false ? 'btn-ghost' : 'btn-primary')}>
              {action.label}
            </a>
          ) : (
            <button
              type='button'
              onClick={action.onClick}
              className={cn('btn btn-sm', action.primary === false ? 'btn-ghost' : 'btn-primary')}>
              {action.label}
            </button>
          )}
        </div>
      )}
      {onDismiss && (
        <button
          type='button'
          onClick={onDismiss}
          className='btn btn-ghost btn-sm ml-auto'
          aria-label='Dismiss'>
          &times;
        </button>
      )}
    </div>
  );
};

export interface NetworkErrorAlertProps {
  onRetry: () => void;
  message?: string;
}

export const NetworkErrorAlert = ({ onRetry, message }: NetworkErrorAlertProps) => (
  <div className='alert alert-error' role='alert'>
    <PiWarningCircle className='icon-sm shrink-0' aria-hidden='true' />
    <div>
      <h3 className='text-sm font-semibold'>Network error</h3>
      <p className='text-sm'>
        {message ?? 'Unable to reach the training API. Please check your connection.'}
      </p>
    </div>
    <button type='button' onClick={onRetry} className='btn btn-neutral btn-sm'>
      Retry
    </button>
  </div>
);
