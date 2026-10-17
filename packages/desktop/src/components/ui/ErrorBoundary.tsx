'use client';

import { Component, type ErrorInfo, type ReactNode } from 'react';
import { ApiAlert } from '@/components/ui/ApiAlert';

export interface ErrorBoundaryProps {
  /** The subtree to protect. */
  children: ReactNode;
  /** A short heading shown when a render error is caught. */
  title?: string;
  /** Plain-language guidance shown under the heading. */
  guidance?: string;
  /** Rendered instead of the default alert when an error is caught. */
  fallback?: (error: Error, reset: () => void) => ReactNode;
}

interface ErrorBoundaryState {
  error: Error | null;
}

/**
 * Catch render-time errors in a subtree and show the studio's own alert.
 *
 * React error boundaries must be class components; this wrapper keeps the
 * fallback consistent with `ApiErrorAlert` (FEATURE-18) so a render crash looks
 * like every other failure rather than the framework error page.
 */
export class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  state: ErrorBoundaryState = { error: null };

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { error };
  }

  componentDidCatch(error: Error, info: ErrorInfo): void {
    // Keep the trace in the console for local debugging; never render it.
    console.error('Render error caught by ErrorBoundary', error, info.componentStack);
  }

  private reset = (): void => {
    this.setState({ error: null });
  };

  render(): ReactNode {
    const { error } = this.state;
    if (!error) {
      return this.props.children;
    }

    if (this.props.fallback) {
      return this.props.fallback(error, this.reset);
    }

    return (
      <ApiAlert
        variant='error'
        title={this.props.title ?? 'Something went wrong'}
        message={error.message || 'An unexpected error occurred while rendering this view.'}
        guidance={this.props.guidance ?? 'Reload the page or try again.'}
        action={{ label: 'Try again', onClick: this.reset, primary: true }}
      />
    );
  }
}
