'use client';

import React, { useRef, useState, useEffect, useCallback } from 'react';
import { cn } from '@/lib/cn';
import { PiDownload } from 'react-icons/pi';
import { svgToPngBlob } from '@/lib/svgToPng';
import { createPortal } from 'react-dom';

interface ChartDownloadMenuProps {
  children: React.ReactElement;
  chartKey: string;
  jobId: string;
  disabled?: boolean;
}

export const ChartDownloadMenu = ({
  children,
  chartKey,
  jobId,
  disabled = false,
}: ChartDownloadMenuProps) => {
  const wrapperRef = useRef<HTMLDivElement>(null);
  const buttonRef = useRef<HTMLButtonElement>(null);
  const dropdownRef = useRef<HTMLDivElement>(null);
  const [isOpen, setIsOpen] = useState(false);

  const getSvgElement = useCallback((): SVGSVGElement | null => {
    if (!wrapperRef.current) return null;
    return wrapperRef.current.querySelector('svg');
  }, []);

  const downloadSvg = useCallback(() => {
    const svgElement = getSvgElement();
    if (!svgElement) return;
    const svgMarkup = new XMLSerializer().serializeToString(svgElement);
    const blob = new Blob([svgMarkup], { type: 'image/svg+xml' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `${jobId}-${chartKey}.svg`;
    a.click();
    URL.revokeObjectURL(url);
    setIsOpen(false);
  }, [getSvgElement, chartKey, jobId]);

  const downloadPng = useCallback(async () => {
    const svgElement = getSvgElement();
    if (!svgElement) return;
    try {
      const blob = await svgToPngBlob(svgElement);
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `${jobId}-${chartKey}.png`;
      a.click();
      URL.revokeObjectURL(url);
    } catch (error) {
      console.error('PNG download failed:', error);
      alert('Failed to download PNG. See console for details.');
    }
    setIsOpen(false);
  }, [getSvgElement, chartKey, jobId]);

  // Close on Escape
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape' && isOpen) {
        setIsOpen(false);
        buttonRef.current?.focus();
      }
    };
    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [isOpen]);

  // Click outside to close
  useEffect(() => {
    if (!isOpen) return;
    const handleClickOutside = (event: MouseEvent) => {
      if (
        wrapperRef.current &&
        !wrapperRef.current.contains(event.target as Node) &&
        buttonRef.current &&
        !buttonRef.current.contains(event.target as Node) &&
        dropdownRef.current &&
        !dropdownRef.current.contains(event.target as Node)
      ) {
        setIsOpen(false);
      }
    };
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, [isOpen]);

  // Position dropdown relative to button
  useEffect(() => {
    if (!isOpen || !buttonRef.current || !dropdownRef.current) return;
    const buttonRect = buttonRef.current.getBoundingClientRect();
    const dropdown = dropdownRef.current;
    dropdown.style.position = 'fixed';
    dropdown.style.top = `${buttonRect.bottom + 4}px`;
    dropdown.style.right = `${window.innerWidth - buttonRect.right + 4}px`;
    dropdown.style.zIndex = '50';
  }, [isOpen]);

  const dropdownContent = (
    <div
      ref={dropdownRef}
      className='dropdown dropdown-end bg-base-100 rounded-box border-base-300 w-40 border shadow-lg'>
      <ul className='menu p-1' role='menu'>
        <li role='menuitem'>
          <button
            className='hover:bg-base-200 w-full rounded px-3 py-1.5 text-left text-sm'
            onClick={downloadSvg}
            disabled={disabled}
            type='button'>
            Download SVG
          </button>
        </li>
        <li role='menuitem'>
          <button
            className='hover:bg-base-200 w-full rounded px-3 py-1.5 text-left text-sm'
            onClick={downloadPng}
            disabled={disabled}
            type='button'>
            Download PNG (2×)
          </button>
        </li>
      </ul>
    </div>
  );

  return (
    <div
      ref={wrapperRef}
      className={cn('relative inline-block', disabled && 'pointer-events-none opacity-50')}>
      {children}
      <button
        ref={buttonRef}
        className={cn(
          'btn btn-ghost btn-xs hover:bg-base-200 focus:ring-primary absolute top-4 right-4 rounded p-1 focus:ring-2 focus:outline-none',
          disabled && 'pointer-events-none opacity-50',
        )}
        aria-label={`Download options for ${chartKey}`}
        aria-expanded={isOpen}
        aria-haspopup='menu'
        onClick={(e) => {
          e.preventDefault();
          e.stopPropagation();
          if (!disabled) setIsOpen((prev) => !prev);
        }}
        type='button'>
        <PiDownload className='icon-sm' aria-hidden='true' />
      </button>
      {isOpen && createPortal(dropdownContent, document.body)}
    </div>
  );
};
