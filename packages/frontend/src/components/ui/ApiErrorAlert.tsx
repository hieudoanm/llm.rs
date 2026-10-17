'use client';

import { ApiAlert, type AlertAction } from '@/components/ui/ApiAlert';
import { ApiError } from '@/lib/api';
import { errorDetails, guidanceFor } from '@/lib/errors';

export interface ApiErrorAlertProps {
  /** The error that was raised. */
  error: unknown;
  /** Called when the user asks to try again. */
  onRetry?: () => void;
  /**
   * The label of the retry action; the action is hidden without a handler.
   * Defaults to the catalogue label of the error.
   */
  retryLabel?: string;
  className?: string;
}

/**
 * Render an error with the guidance of the central catalogue.
 *
 * The transport code and the backend code are both kept, so the details
 * disclosure can quote the raw code the user should report.
 *
 * The catalogue decides which remedy is offered: an error that carries a
 * navigation remedy (for example a deleted job) shows that link, because
 * retrying the request cannot help. Every other error offers the retry
 * handler, labelled with the catalogue action when it defines one.
 */
export const ApiErrorAlert = ({ error, onRetry, retryLabel, className }: ApiErrorAlertProps) => {
  const apiError = error instanceof ApiError ? error : null;
  const guidance = guidanceFor(apiError);
  const details = errorDetails(apiError);

  const action: AlertAction | undefined = guidance.actionHref
    ? {
        label: guidance.action ?? 'Go on',
        href: guidance.actionHref,
        primary: guidance.retryable,
      }
    : onRetry
      ? {
          label: retryLabel ?? guidance.action ?? 'Retry',
          onClick: onRetry,
          primary: guidance.retryable,
        }
      : undefined;

  return (
    <ApiAlert
      variant='error'
      title={guidance.title}
      message={error instanceof Error ? error.message : 'The request failed.'}
      guidance={guidance.message}
      action={action}
      details={details.length ? details : undefined}
      requestId={apiError?.requestId}
      className={className}
    />
  );
};
