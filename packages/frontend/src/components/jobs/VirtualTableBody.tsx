'use client';

import { useRef, type ReactNode } from 'react';
import { useVirtualizer } from '@tanstack/react-virtual';

export interface VirtualTableBodyProps {
  /** The number of rows to render. */
  count: number;
  /** Estimated row height, in pixels; measured per row once rendered. */
  estimateSize?: number;
  /** A fixed pixel height for the scroll container. */
  maxHeight?: number;
  /** The `<tr>` elements, one per index. */
  renderRow: (index: number) => ReactNode;
  /** Extra rows pinned below the virtualised window (e.g. an editor). */
  footer?: ReactNode;
}

/**
 * A windowed `<tbody>` (FEATURE-18).
 *
 * The job history can hold hundreds of rows; rendering them all costs layout
 * work on every poll. Only the rows in view are mounted, positioned with a
 * `translateY` so the native table semantics (columns, headers, `role`) are
 * preserved for assistive technology.
 */
export const VirtualTableBody = ({
  count,
  estimateSize = 57,
  maxHeight = 640,
  renderRow,
  footer,
}: VirtualTableBodyProps) => {
  const scrollRef = useRef<HTMLDivElement>(null);
  const virtualizer = useVirtualizer({
    count,
    getScrollElement: () => scrollRef.current,
    estimateSize: () => estimateSize,
    overscan: 8,
  });

  const items = virtualizer.getVirtualItems();
  const paddingTop = items.length > 0 ? items[0].start : 0;
  const paddingBottom =
    items.length > 0 ? virtualizer.getTotalSize() - items[items.length - 1].end : 0;

  return (
    <div
      ref={scrollRef}
      className='overflow-auto'
      style={{ maxHeight }}
      data-testid='virtual-table-scroll'>
      <table className='table'>
        <tbody>
          {paddingTop > 0 && (
            <tr aria-hidden='true'>
              <td style={{ height: paddingTop, padding: 0 }} colSpan={6} />
            </tr>
          )}
          {items.map((item) => (
            <VirtualRow key={item.key} index={item.index} measure={virtualizer.measureElement}>
              {renderRow(item.index)}
            </VirtualRow>
          ))}
          {paddingBottom > 0 && (
            <tr aria-hidden='true'>
              <td style={{ height: paddingBottom, padding: 0 }} colSpan={6} />
            </tr>
          )}
          {footer}
        </tbody>
      </table>
    </div>
  );
};

interface VirtualRowProps {
  index: number;
  measure: (node: Element | null) => void;
  children: ReactNode;
}

/**
 * One measured row.
 *
 * The `data-index` attribute is how the virtualizer measures the real height
 * after mount, which keeps rows of varying tag counts correct.
 */
const VirtualRow = ({ index, measure, children }: VirtualRowProps) => (
  <tr ref={measure} data-index={index}>
    {children}
  </tr>
);
