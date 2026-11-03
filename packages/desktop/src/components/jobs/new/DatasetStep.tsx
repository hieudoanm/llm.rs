'use client';

import { Button } from '@/components/ui/Button';
import { FileDrop } from '@/components/ui/FileDrop';
import { Dataset } from '@/app/jobs/new/_lib/types';
import { UseMutationResult } from '@tanstack/react-query';
import type { ValidationIssue } from '@/app/jobs/new/_lib/validation';

type Props = {
  dataset: Dataset | null;
  uploadError: string | null;
  uploadMutation: UseMutationResult<Dataset, Error, FormData>;
  onUpload: (file: File) => void;
  onChangeFile: () => void;
  onContinue: () => void;
  /** The live validation issues of this step. */
  issues?: ValidationIssue[];
  /** True when the user asked to see the validation summary. */
  showIssues?: boolean;
};

export const DatasetStep = ({
  dataset,
  uploadError,
  uploadMutation,
  onUpload,
  onChangeFile,
  onContinue,
  issues = [],
  showIssues = false,
}: Props) => {
  const fileIssue = issues.find((issue) => issue.field === 'dataset-file');
  const showFileIssue = showIssues && fileIssue;

  return (
    <div className='wizard-step card bg-base-100 mb-6 p-6 shadow'>
      <h2 className='text-text-primary mb-4 text-lg font-semibold'>1. Dataset</h2>
      <div
        id='dataset-file'
        tabIndex={-1}
        aria-describedby={showFileIssue ? 'dataset-file-error' : undefined}
        aria-invalid={showFileIssue ? true : undefined}>
        <FileDrop onFileSelected={onUpload} disabled={uploadMutation.isPending} />
      </div>
      {showFileIssue && (
        <p id='dataset-file-error' className='text-error mt-2 text-sm'>
          {fileIssue.message}
        </p>
      )}
      {uploadError && (
        <p className='text-error mt-2 text-sm' role='alert'>
          {uploadError}
        </p>
      )}
      {uploadMutation.isPending && (
        <div className='mt-4 flex items-center justify-center gap-3'>
          <span className='loading loading-spinner loading-sm' aria-hidden='true' />
          <span className='text-text-secondary text-sm'>Uploading CSV…</span>
        </div>
      )}
      {dataset && (
        <div className='mt-6'>
          <div className='mb-4 flex items-center justify-between gap-4'>
            <p className='text-text-primary text-sm font-medium'>
              {dataset.filename}{' '}
              <span className='text-text-tertiary'>· {dataset.rowCount.toLocaleString()} rows</span>
            </p>
            <Button variant='outline' size='sm' onClick={onChangeFile}>
              Change file
            </Button>
          </div>
          <div className='border-border overflow-x-auto rounded-lg border'>
            <table className='table'>
              <thead>
                <tr>
                  <th>Column</th>
                  <th>Type</th>
                  <th>Missing</th>
                  <th>Sample values</th>
                </tr>
              </thead>
              <tbody>
                {dataset.columns.map((col) => (
                  <tr key={col.name}>
                    <td className='font-mono text-sm'>{col.name}</td>
                    <td className='font-mono text-xs'>{col.inferredDtype}</td>
                    <td className='font-mono text-xs'>{col.nullCount}</td>
                    <td className='text-text-secondary max-w-xs truncate text-xs'>
                      {col.sampleValues.map((value) => String(value)).join(', ')}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          <div className='mt-6 flex justify-end'>
            <Button onClick={onContinue}>Continue to mapping</Button>
          </div>
        </div>
      )}
    </div>
  );
};
