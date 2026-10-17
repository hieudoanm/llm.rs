'use client';

import { Button } from '@/components/ui/Button';
import { cn } from '@/lib/cn';
import { Dataset, TaskType } from '@/app/jobs/new/_lib/types';
import { TASKS, TASK_GROUPS } from '@/app/jobs/new/_lib/constants';
import { RLColumnsField } from './RLColumnsField';
import type { ValidationIssue } from '@/app/jobs/new/_lib/validation';

type Props = {
  dataset: Dataset | null;
  featureColumns: string[];
  onToggleFeature: (name: string) => void;
  taskType: TaskType;
  onTaskChange: (next: TaskType) => void;
  requiresLabel: boolean;
  labelColumn: string;
  onLabelChange: (value: string) => void;
  labelledRows: number;
  unlabelledRows: number;
  isReinforcement: boolean;
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
  onBack: () => void;
  onContinue: () => void;
  /** The live validation issues of this step. */
  issues?: ValidationIssue[];
  /** True when the user asked to see the validation summary. */
  showIssues?: boolean;
};

export const MappingStep = ({
  dataset,
  featureColumns,
  onToggleFeature,
  taskType,
  onTaskChange,
  requiresLabel,
  labelColumn,
  onLabelChange,
  labelledRows,
  unlabelledRows,
  isReinforcement,
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
  onBack,
  onContinue,
  issues = [],
  showIssues = false,
}: Props) => {
  const featureIssue = issues.find((issue) => issue.field === 'feature-columns');
  const labelIssue = issues.find((issue) => issue.field === 'label-column');
  const showFeatureIssue = showIssues && featureIssue;
  const showLabelIssue = showIssues && labelIssue;

  return (
    <div className='wizard-step card bg-base-100 mb-6 p-6 shadow'>
      <h2 className='text-text-primary mb-4 text-lg font-semibold'>2. Mapping</h2>
      <div className='grid gap-4'>
        <fieldset
          id='feature-columns'
          tabIndex={-1}
          aria-describedby={showFeatureIssue ? 'feature-columns-error' : undefined}
          aria-invalid={showFeatureIssue ? true : undefined}>
          <legend className='label-text mb-2'>Feature columns</legend>
          <div className='flex flex-wrap gap-2'>
            {dataset?.columns.map((col) => (
              <label
                key={col.name}
                className={cn(
                  'btn btn-sm',
                  featureColumns.includes(col.name) ? 'btn-primary' : 'btn-outline',
                )}>
                <input
                  type='checkbox'
                  className='checkbox checkbox-sm'
                  checked={featureColumns.includes(col.name)}
                  onChange={() => onToggleFeature(col.name)}
                />
                {col.name}
              </label>
            ))}
          </div>
        </fieldset>
        {showFeatureIssue && (
          <p id='feature-columns-error' className='text-error text-sm'>
            {featureIssue.message}
          </p>
        )}
        <div className='form-control'>
          <label className='label' htmlFor='task-type'>
            <span className='label-text'>Task</span>
          </label>
          <select
            id='task-type'
            className='select select-bordered w-full'
            value={taskType}
            onChange={(e) => onTaskChange(e.target.value as TaskType)}>
            {TASK_GROUPS.map((group) => (
              <optgroup key={group.label} label={group.label}>
                {group.tasks.map((value) => (
                  <option key={value} value={value}>
                    {TASKS.find((task) => task.value === value)?.label ?? value}
                  </option>
                ))}
              </optgroup>
            ))}
          </select>
          <span className='text-text-tertiary mt-2 text-xs'>
            {TASKS.find((task) => task.value === taskType)?.help}
          </span>
        </div>
        {isReinforcement && (
          <RLColumnsField
            dataset={dataset}
            featureColumns={featureColumns}
            rlActionColumn={rlActionColumn}
            setRlActionColumn={setRlActionColumn}
            rlRewardColumn={rlRewardColumn}
            setRlRewardColumn={setRlRewardColumn}
            rlDoneColumn={rlDoneColumn}
            setRlDoneColumn={setRlDoneColumn}
            rlEpisodeColumn={rlEpisodeColumn}
            setRlEpisodeColumn={setRlEpisodeColumn}
            rlNextStateColumns={rlNextStateColumns}
            toggleNextState={toggleNextState}
          />
        )}
        {requiresLabel && (
          <div className='form-control'>
            <label className='label' htmlFor='label-column'>
              <span className='label-text'>Label column</span>
            </label>
            <select
              id='label-column'
              className='select select-bordered w-full'
              value={labelColumn}
              aria-invalid={showLabelIssue ? true : undefined}
              aria-describedby={showLabelIssue ? 'label-column-error' : undefined}
              onChange={(e) => onLabelChange(e.target.value)}>
              <option value=''>Select label column</option>
              {dataset?.columns.map((col) => (
                <option key={col.name} value={col.name}>
                  {col.name}
                </option>
              ))}
            </select>
            {showLabelIssue && (
              <p id='label-column-error' className='text-error mt-2 text-sm'>
                {labelIssue.message}
              </p>
            )}
            {taskType === 'semi_supervised' && (
              <span className='text-text-tertiary mt-2 text-xs'>
                Leave a label cell empty to treat that row as unlabelled.
              </span>
            )}
          </div>
        )}
        {taskType === 'regression' && (
          <p className='text-text-tertiary text-xs'>
            For regression, the label column must be numeric.
          </p>
        )}
        {taskType === 'semi_supervised' && labelColumn && (
          <div className='bg-base-200 rounded-lg p-4 text-sm'>
            <p className='text-text-primary font-medium'>
              {labelledRows.toLocaleString()} labelled · {unlabelledRows.toLocaleString()}{' '}
              unlabelled
            </p>
            <p className='text-text-secondary mt-1 text-xs'>
              Empty, whitespace-only, NaN, and null cells are unlabelled. At least 10 labelled rows,
              2 classes, and 2 labelled rows per class are required.
            </p>
            {labelledRows < 10 && (
              <p className='text-error mt-2 text-xs'>
                This column has fewer than 10 labelled rows.
              </p>
            )}
          </div>
        )}
        {(taskType === 'clustering' || taskType === 'anomaly_detection') && (
          <p className='text-text-tertiary text-xs'>
            No label column is used. All rows are fitted and scored in place.
          </p>
        )}
        {isReinforcement && (
          <p className='text-text-tertiary text-xs'>
            No label column is used. Map the logged transitions below.
          </p>
        )}
      </div>
      <div className='mt-6 flex justify-between'>
        <Button variant='outline' onClick={onBack}>
          Back
        </Button>
        <Button onClick={onContinue}>Continue to model</Button>
      </div>
    </div>
  );
};
