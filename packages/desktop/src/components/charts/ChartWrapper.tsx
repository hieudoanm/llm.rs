'use client';

import { cn } from '@/lib/cn';

interface ChartWrapperProps {
  /** Chart title (used as heading and aria-label). */
  title: string;
  /** One-sentence caption explaining what the chart shows. */
  caption: string;
  /** Optional CSS class for the wrapper card. */
  className?: string;
  /** Children rendered inside the SVG area. */
  children: React.ReactNode;
  /** Visually-hidden table for accessibility fallback. */
  fallbackTable?: React.ReactNode;
  /** SVG viewBox dimensions (width, height). */
  viewBox?: { width: number; height: number };
}

export const ChartWrapper = ({
  title,
  caption,
  className,
  children,
  fallbackTable,
  viewBox = { width: 500, height: 300 },
}: ChartWrapperProps) => {
  return (
    <div className={cn('card bg-base-100 shadow', className)}>
      <div className='card-body'>
        <h3 className='card-title text-text-primary'>{title}</h3>
        <p className='text-text-secondary mb-3 text-sm'>{caption}</p>
        <div
          className='relative'
          style={{
            width: '100%',
            maxWidth: viewBox.width,
            aspectRatio: `${viewBox.width} / ${viewBox.height}`,
          }}>
          <svg
            viewBox={`0 0 ${viewBox.width} ${viewBox.height}`}
            preserveAspectRatio='xMidYMid meet'
            className='h-full w-full'
            role='img'
            aria-label={title}>
            <title>{title}</title>
            {children}
          </svg>
        </div>
        {fallbackTable && (
          <details className='mt-3'>
            <summary className='text-text-secondary cursor-pointer text-sm select-none'>
              Show values as table
            </summary>
            <div className='mt-2 overflow-x-auto'>{fallbackTable}</div>
          </details>
        )}
      </div>
    </div>
  );
};
