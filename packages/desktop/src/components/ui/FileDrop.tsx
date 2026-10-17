'use client';

import { useCallback, useId, useState } from 'react';
import type { ChangeEvent, DragEvent } from 'react';
import { cn } from '@/lib/cn';
import { PiUpload } from 'react-icons/pi';

export interface FileDropProps {
  onFileSelected: (file: File) => void;
  accept?: string;
  disabled?: boolean;
  maxSize?: number;
}

const ACCEPT_CSV = '.csv';
const MAX_SIZE_MB = 10;

export const FileDrop = ({
  onFileSelected,
  accept = ACCEPT_CSV,
  disabled = false,
  maxSize = MAX_SIZE_MB,
}: FileDropProps) => {
  const [dragOver, setDragOver] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const errorId = useId();

  const validateAndEmit = useCallback(
    (file: File) => {
      setError(null);
      if (!new RegExp(`\\.(${accept.replace('.', '')})$`, 'i').test(file.name)) {
        setError('CSV file only');
        return;
      }
      if (maxSize && file.size > maxSize * 1024 * 1024) {
        setError(`File exceeds ${maxSize} MB`);
        return;
      }
      onFileSelected(file);
    },
    [accept, maxSize, onFileSelected],
  );

  const handleDrop = useCallback(
    (e: DragEvent<HTMLDivElement>) => {
      e.preventDefault();
      setDragOver(false);
      const file = e.dataTransfer.files[0];
      if (file) validateAndEmit(file);
    },
    [validateAndEmit],
  );

  const handleDragOver = useCallback((e: DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    setDragOver(true);
  }, []);

  const handleDragLeave = useCallback(() => {
    setDragOver(false);
  }, []);

  const handleFileInputChange = useCallback(
    (e: ChangeEvent<HTMLInputElement>) => {
      const file = e.target.files?.[0];
      if (file) validateAndEmit(file);
      e.target.value = '';
    },
    [validateAndEmit],
  );

  return (
    <div
      className={cn(
        'border-border relative rounded-2xl border-2 border-dashed p-10 text-center transition-colors duration-150',
        dragOver ? 'border-primary bg-primary-muted' : '',
        disabled && 'pointer-events-none opacity-50',
      )}
      onDragOver={handleDragOver}
      onDragLeave={handleDragLeave}
      onDrop={handleDrop}>
      <input
        type='file'
        accept={accept}
        aria-label='Upload CSV'
        aria-invalid={error ? true : undefined}
        aria-describedby={error ? errorId : undefined}
        className='absolute inset-0 h-full w-full cursor-pointer opacity-0'
        onChange={handleFileInputChange}
        disabled={disabled}
      />
      <div className='pointer-events-none flex flex-col items-center justify-center gap-3'>
        <div className='bg-surface-raised rounded-full p-4'>
          <PiUpload className='icon-lg' aria-hidden='true' />
        </div>
        <p className='text-text-primary text-sm font-medium'>Drag &amp; drop your CSV here</p>
        <p className='text-text-secondary text-xs'>
          or <span className='text-primary font-medium'>browse</span>
        </p>
        <p className='text-text-tertiary text-[10px] tracking-wider uppercase'>CSV only</p>
      </div>
      {error && (
        <p
          id={errorId}
          role='alert'
          className='text-error-fg bg-error-bg absolute top-2 left-1/2 -translate-x-1/2 rounded px-2 py-1 text-[10px]'>
          {error}
        </p>
      )}
    </div>
  );
};
