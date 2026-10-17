'use client';

import { cn } from '@/lib/cn';
import { Dataset } from '@/app/jobs/new/_lib/types';

type RlColumnSelectProps = {
  id: string;
  label: string;
  value: string;
  onChange: (value: string) => void;
  columns: Dataset['columns'];
  required?: boolean;
};

const RlColumnSelect = ({ id, label, value, onChange, columns, required }: RlColumnSelectProps) => (
  <div className='form-control'>
    <label className='label' htmlFor={id}>
      <span className='label-text'>{label}</span>
    </label>
    <select
      id={id}
      className='select select-bordered w-full'
      value={value}
      required={required}
      onChange={(e) => onChange(e.target.value)}>
      <option value=''>Select column</option>
      {columns.map((col) => (
        <option key={col.name} value={col.name}>
          {col.name}
        </option>
      ))}
    </select>
  </div>
);

export type RLColumnsFieldProps = {
  dataset: Dataset | null;
  featureColumns: string[];
  rlActionColumn: string;
  setRlActionColumn: (value: string) => void;
  rlRewardColumn: string;
  setRlRewardColumn: (value: string) => void;
  rlDoneColumn: string;
  setRlDoneColumn: (value: string) => void;
  rlEpisodeColumn: string;
  setRlEpisodeColumn: (value: string) => void;
  rlNextStateColumns: string[];
  toggleNextState: (name: string) => void;
};

export const RLColumnsField = ({
  dataset,
  featureColumns,
  rlActionColumn,
  setRlActionColumn,
  rlRewardColumn,
  setRlRewardColumn,
  rlDoneColumn,
  setRlDoneColumn,
  rlEpisodeColumn,
  setRlEpisodeColumn,
  rlNextStateColumns,
  toggleNextState,
}: RLColumnsFieldProps) => (
  <fieldset className='border-border rounded-lg border p-4'>
    <legend className='label-text px-1'>Experience columns</legend>
    <div className='grid gap-4 sm:grid-cols-2'>
      <RlColumnSelect
        id='rl-action'
        label='Action'
        value={rlActionColumn}
        onChange={setRlActionColumn}
        columns={dataset?.columns ?? []}
        required
      />
      <RlColumnSelect
        id='rl-reward'
        label='Reward'
        value={rlRewardColumn}
        onChange={setRlRewardColumn}
        columns={dataset?.columns ?? []}
        required
      />
      <RlColumnSelect
        id='rl-done'
        label='Done (optional)'
        value={rlDoneColumn}
        onChange={setRlDoneColumn}
        columns={dataset?.columns ?? []}
      />
      <RlColumnSelect
        id='rl-episode'
        label='Episode (optional)'
        value={rlEpisodeColumn}
        onChange={setRlEpisodeColumn}
        columns={dataset?.columns ?? []}
      />
    </div>
    <div className='mt-4'>
      <p className='label-text mb-2'>
        Next state — one column per state feature ({featureColumns.length} selected)
      </p>
      <div className='flex flex-wrap gap-2'>
        {dataset?.columns.map((col) => (
          <label
            key={col.name}
            className={cn(
              'btn btn-sm',
              rlNextStateColumns.includes(col.name) ? 'btn-primary' : 'btn-outline',
            )}>
            <input
              type='checkbox'
              className='checkbox checkbox-sm'
              checked={rlNextStateColumns.includes(col.name)}
              onChange={() => toggleNextState(col.name)}
            />
            {col.name}
          </label>
        ))}
      </div>
      <span className='text-text-tertiary mt-2 block text-xs'>
        Select the same number of next-state columns as state features, in order. When the episode
        column is set, the next state can instead be derived from CSV order.
      </span>
    </div>
  </fieldset>
);
