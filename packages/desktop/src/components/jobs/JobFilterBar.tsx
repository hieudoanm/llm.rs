'use client';

import { useState } from 'react';
import { cn } from '@/lib/cn';
import { PiFunnel, PiSortAscending, PiX } from 'react-icons/pi';
import {
  JOB_STATUSES,
  JOB_TASK_TYPES,
  type JobFilters,
  filterChips,
  nextSort,
  removeChip,
  toggleFilterValue,
} from '@/lib/jobs';

interface JobFilterBarProps {
  filters: JobFilters;
  onChange: (filters: JobFilters) => void;
  /** The algorithm ids offered by the filter bar. */
  algorithmIds?: string[];
  /** The tags already used in the workspace. */
  knownTags?: string[];
}

const TASK_LABELS: Record<string, string> = {
  classification: 'Classification',
  regression: 'Regression',
  semi_supervised: 'Semi-supervised',
  clustering: 'Clustering',
  anomaly_detection: 'Anomaly detection',
  reinforcement_learning: 'Reinforcement learning',
};

interface MultiSelectProps {
  label: string;
  options: string[];
  selected: string[];
  /** Renders the option label; defaults to the raw value. */
  format?: (value: string) => string;
  onToggle: (value: string) => void;
}

const MultiSelect = ({ label, options, selected, format, onToggle }: MultiSelectProps) => {
  const [open, setOpen] = useState(false);
  if (options.length === 0) return null;

  return (
    <div className='dropdown'>
      <button
        type='button'
        className={cn('btn btn-sm', selected.length > 0 && 'btn-primary')}
        onClick={() => setOpen((value) => !value)}
        aria-label={`Filter by ${label.toLowerCase()}`}
        aria-expanded={open}
        aria-haspopup='true'>
        {label}
        {selected.length > 0 && (
          <span className='badge badge-sm badge-neutral'>{selected.length}</span>
        )}
      </button>
      {open && (
        <ul
          className='dropdown-content menu rounded-box border-border bg-base-100 z-10 mt-1 w-56 border p-2 shadow'
          role='group'
          aria-label={label}>
          {options.map((option) => (
            <li key={option}>
              <label className='flex cursor-pointer items-center gap-2'>
                <input
                  type='checkbox'
                  className='checkbox checkbox-sm'
                  checked={selected.includes(option)}
                  onChange={() => onToggle(option)}
                />
                <span>{format ? format(option) : option}</span>
              </label>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
};

/**
 * The filter bar of the jobs page.
 *
 * Every control maps to exactly one query parameter, and the active filter
 * set is shown as removable chips below the bar.
 */
export const JobFilterBar = ({
  filters,
  onChange,
  algorithmIds = [],
  knownTags = [],
}: JobFilterBarProps) => {
  const chips = filterChips(filters);

  return (
    <section className='mb-4' aria-label='Job filters'>
      <div className='flex flex-wrap items-center gap-2'>
        <span className='text-text-secondary flex items-center gap-1 text-xs font-medium'>
          <PiFunnel className='icon-sm' aria-hidden='true' />
          Filters
        </span>

        <label className='input input-sm flex items-center gap-2'>
          <span className='sr-only'>Search jobs</span>
          <input
            type='search'
            className='grow'
            placeholder='Search name, tag, or algorithm'
            value={filters.q}
            onChange={(event) => onChange({ ...filters, q: event.target.value })}
          />
        </label>

        <MultiSelect
          label='Status'
          options={[...JOB_STATUSES]}
          selected={filters.status}
          onToggle={(value) => onChange(toggleFilterValue(filters, 'status', value))}
        />

        <MultiSelect
          label='Task'
          options={[...JOB_TASK_TYPES]}
          selected={filters.taskType}
          format={(value) => TASK_LABELS[value] ?? value}
          onToggle={(value) => onChange(toggleFilterValue(filters, 'taskType', value))}
        />

        <MultiSelect
          label='Algorithm'
          options={algorithmIds}
          selected={filters.algorithmId}
          onToggle={(value) => onChange(toggleFilterValue(filters, 'algorithmId', value))}
        />

        <MultiSelect
          label='Tag'
          options={knownTags}
          selected={filters.tag}
          onToggle={(value) => onChange(toggleFilterValue(filters, 'tag', value))}
        />

        <div className='dropdown dropdown-end'>
          <button type='button' tabIndex={0} className='btn btn-sm' aria-label='Sort jobs'>
            <PiSortAscending className='icon-sm' aria-hidden='true' />
            Sort
          </button>
          <ul
            tabIndex={0}
            className='dropdown-content menu rounded-box border-border bg-base-100 z-10 mt-1 w-48 border p-2 shadow'>
            {(['createdAt', 'updatedAt', 'status', 'algorithmId'] as const).map((field) => (
              <li key={field}>
                <button
                  type='button'
                  onClick={() => onChange({ ...filters, sort: nextSort(filters.sort, field) })}>
                  {field === 'createdAt'
                    ? 'Created'
                    : field === 'updatedAt'
                      ? 'Updated'
                      : field === 'status'
                        ? 'Status'
                        : 'Algorithm'}
                </button>
              </li>
            ))}
          </ul>
        </div>

        {chips.length > 0 && (
          <button
            type='button'
            className='btn btn-ghost btn-sm'
            onClick={() =>
              onChange({
                status: [],
                taskType: [],
                algorithmId: [],
                tag: [],
                q: '',
                createdFrom: '',
                createdTo: '',
                sort: '',
              })
            }>
            Clear all
          </button>
        )}
      </div>

      {chips.length > 0 && (
        <ul className='mt-2 flex flex-wrap items-center gap-2' aria-label='Active filters'>
          {chips.map((chip) => (
            <li key={`${chip.group}-${chip.value}-${chip.label}`}>
              <button
                type='button'
                className='badge badge-outline gap-1'
                onClick={() => onChange(removeChip(filters, chip))}
                aria-label={`Remove filter ${chip.label}`}>
                {chip.label}
                <PiX className='icon-xs' aria-hidden='true' />
              </button>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
};
