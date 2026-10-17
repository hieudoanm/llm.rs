'use client';

import { Button } from '@/components/ui/Button';
import { ApiErrorAlert } from '@/components/ui/ApiErrorAlert';
import { HyperparameterForm } from '@/components/jobs/new/HyperparameterForm';
import {
  Dataset,
  Algorithm,
  TaskType,
  ExportFormat,
  HyperparameterField,
} from '@/app/jobs/new/_lib/types';
import { UseMutationResult } from '@tanstack/react-query';
import { JobAccepted } from '@/app/jobs/new/_lib/types';
import type { ValidationIssue } from '@/app/jobs/new/_lib/validation';

type Props = {
  dataset: Dataset | null;
  taskType: TaskType;
  algorithms: Algorithm[];
  loadingAlgorithms: boolean;
  algorithmId: string;
  handleAlgorithmChange: (value: string) => void;
  selectedAlgorithm?: Algorithm;
  hyperparameterFields: HyperparameterField[];
  hyperparameters: Record<string, string | number | boolean>;
  setHyperparameter: (key: string, value: string | number | boolean) => void;
  contaminationMode: 'auto' | 'custom';
  setContaminationMode: (mode: 'auto' | 'custom') => void;
  availableExports: string[];
  exportState: Record<ExportFormat, boolean>;
  setExport: Record<ExportFormat, (checked: boolean) => void>;
  createJobMutation: UseMutationResult<JobAccepted, Error, void>;
  onBack: () => void;
  onStart: () => void;
  /** The live validation issues of this step. */
  issues?: ValidationIssue[];
  /** True when the user asked to see the validation summary. */
  showIssues?: boolean;
};

export const ModelStep = ({
  dataset,
  algorithms,
  loadingAlgorithms,
  algorithmId,
  handleAlgorithmChange,
  selectedAlgorithm,
  hyperparameterFields,
  hyperparameters,
  setHyperparameter,
  contaminationMode,
  setContaminationMode,
  availableExports,
  exportState,
  setExport,
  createJobMutation,
  onBack,
  onStart,
  issues = [],
  showIssues = false,
}: Props) => {
  const algorithmIssue = issues.find((issue) => issue.field === 'algorithm');
  const showAlgorithmIssue = showIssues && algorithmIssue;

  return (
    <div className='wizard-step card bg-base-100 mb-6 p-6 shadow'>
      <h2 className='text-text-primary mb-4 text-lg font-semibold'>3. Model</h2>
      {loadingAlgorithms ? (
        <div className='flex items-center justify-center py-8'>
          <span className='loading loading-spinner loading-lg' aria-hidden='true' />
        </div>
      ) : (
        <div className='grid gap-4'>
          <div className='form-control'>
            <label className='label' htmlFor='algorithm'>
              <span className='label-text'>Algorithm</span>
            </label>
            <select
              id='algorithm'
              className='select select-bordered w-full'
              value={algorithmId}
              aria-invalid={showAlgorithmIssue ? true : undefined}
              aria-describedby={showAlgorithmIssue ? 'algorithm-error' : undefined}
              onChange={(e) => handleAlgorithmChange(e.target.value)}>
              <option value=''>Select algorithm</option>
              {(() => {
                // Group algorithms by task type for the dropdown
                const taskOrder: TaskType[] = [
                  'classification',
                  'regression',
                  'semi_supervised',
                  'clustering',
                  'anomaly_detection',
                  'reinforcement_learning',
                ];
                const groups = new Map<TaskType, Algorithm[]>();
                for (const algo of algorithms) {
                  const list = groups.get(algo.taskType) ?? [];
                  list.push(algo);
                  groups.set(algo.taskType, list);
                }
                return taskOrder
                  .filter((t) => groups.has(t))
                  .map((taskType) => (
                    <optgroup key={taskType} label={taskType.replace('_', ' ').toUpperCase()}>
                      {groups.get(taskType)!.map((algo) => (
                        <option key={algo.algorithmId} value={algo.algorithmId}>
                          {algo.name} — {algo.description}
                        </option>
                      ))}
                    </optgroup>
                  ));
              })()}
            </select>
            {showAlgorithmIssue && (
              <p id='algorithm-error' className='text-error mt-2 text-sm'>
                {algorithmIssue.message}
              </p>
            )}
          </div>
          {selectedAlgorithm && (
            <div className='bg-base-200 rounded-lg p-4'>
              <div className='flex items-center gap-2'>
                <span className='text-sm font-semibold'>{selectedAlgorithm.name}</span>
                <span className='badge badge-ghost badge-sm'>{selectedAlgorithm.taskType}</span>
                <span className='badge badge-outline badge-sm'>{selectedAlgorithm.family}</span>
              </div>
              <p className='text-text-secondary mt-1 text-sm'>{selectedAlgorithm.description}</p>
            </div>
          )}
          {hyperparameterFields.length > 0 && (
            <fieldset className='border-border rounded-lg border p-4'>
              <legend className='label-text px-1'>Hyperparameters</legend>
              <HyperparameterForm
                fields={hyperparameterFields}
                rowCount={dataset?.rowCount ?? 0}
                hyperparameters={hyperparameters}
                setHyperparameter={setHyperparameter}
                issues={issues}
                showIssues={showIssues}
              />
              {algorithmId === 'isolation_forest' && (
                <div className='mt-4'>
                  <div className='form-control'>
                    <label className='label' htmlFor='hp-contamination-mode'>
                      <span className='label-text'>Contamination</span>
                    </label>
                    <select
                      id='hp-contamination-mode'
                      className='select select-bordered w-full'
                      value={contaminationMode}
                      onChange={(e) => {
                        const mode = e.target.value as 'auto' | 'custom';
                        setContaminationMode(mode);
                        setHyperparameter('contamination', mode === 'auto' ? 'auto' : 0.1);
                      }}>
                      <option value='auto'>auto</option>
                      <option value='custom'>Custom rate</option>
                    </select>
                    {contaminationMode === 'custom' && (
                      <input
                        id='hp-contamination'
                        aria-label='Contamination rate'
                        type='number'
                        className='input input-bordered mt-2 w-full'
                        min={0.01}
                        max={0.5}
                        step={0.01}
                        value={String(hyperparameters.contamination ?? '')}
                        aria-invalid={
                          showIssues &&
                          issues.some((e) => e.field === 'hyperparameter-contamination')
                            ? true
                            : undefined
                        }
                        aria-describedby={
                          showIssues &&
                          issues.some((e) => e.field === 'hyperparameter-contamination')
                            ? 'hp-contamination-error'
                            : undefined
                        }
                        onChange={(e) =>
                          setHyperparameter(
                            'contamination',
                            e.target.value === '' ? '' : Number(e.target.value),
                          )
                        }
                      />
                    )}
                    <span className='text-text-tertiary mt-1 text-xs'>
                      Expected share of outliers: auto or 0.01–0.5.
                    </span>
                    {showIssues &&
                      issues
                        .filter((e) => e.field === 'hyperparameter-contamination')
                        .map((issue) => (
                          <p
                            key={issue.message}
                            id='hp-contamination-error'
                            className='text-error mt-1 text-xs'>
                            {issue.message}
                          </p>
                        ))}
                  </div>
                </div>
              )}
            </fieldset>
          )}
          <fieldset>
            <legend className='label-text mb-2'>Extra export formats</legend>
            <div className='flex flex-wrap gap-4'>
              {(['pkl', 'joblib'] as ExportFormat[])
                .filter((format) => availableExports.includes(format))
                .map((format) => (
                  <label key={format} className='flex items-center gap-2'>
                    <input
                      type='checkbox'
                      className='checkbox checkbox-sm'
                      checked={exportState[format]}
                      onChange={(e) => setExport[format](e.target.checked)}
                    />
                    <span className='text-sm'>{format}</span>
                  </label>
                ))}
            </div>
            <p className='text-text-tertiary mt-2 text-xs'>
              ONNX is always exported. Additional formats depend on the selected algorithm.
            </p>
          </fieldset>
        </div>
      )}
      <div className='mt-6 flex justify-between'>
        <Button variant='outline' onClick={onBack}>
          Back
        </Button>
        <Button onClick={onStart} disabled={createJobMutation.isPending}>
          Start training
        </Button>
      </div>
      {createJobMutation.isError && (
        <div className='mt-4'>
          <ApiErrorAlert error={createJobMutation.error} />
        </div>
      )}
    </div>
  );
};
