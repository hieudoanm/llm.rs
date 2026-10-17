'use client';

import { useCallback, useEffect, useState } from 'react';
import { ErrorBoundary } from '@/components/ui/ErrorBoundary';
import { WizardStepper } from '@/components/jobs/new/WizardStepper';
import { DatasetStep } from '@/components/jobs/new/DatasetStep';
import { MappingStep } from '@/components/jobs/new/MappingStep';
import { ModelStep } from '@/components/jobs/new/ModelStep';
import { nextStep, previousStep, stepForDigit } from './_lib/keyboard';
import { useNewJobWizard } from './_lib/hooks';
import type { ValidationIssue } from './_lib/validation';
import type { WizardStep } from './_lib/types';

/** The element id of the control an issue belongs to. */
const FIELD_TARGETS: Record<string, string> = {
  'dataset-file': 'dataset-file',
  'feature-columns': 'feature-columns',
  'label-column': 'label-column',
  algorithm: 'algorithm',
};

const targetFor = (issue: ValidationIssue): string | undefined =>
  FIELD_TARGETS[issue.field] ??
  (issue.field.startsWith('hyperparameter-')
    ? `hp-${issue.field.replace('hyperparameter-', '')}`
    : undefined);

function NewJobWizard() {
  const {
    step,
    setStep,
    dataset,
    setDataset,
    uploadError,
    featureColumns,
    setFeatureColumns,
    taskType,
    labelColumn,
    setLabelColumn,
    onLabelChange,
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
    algorithmId,
    hyperparameters,
    contaminationMode,
    setContaminationMode,
    exportState,
    setExport,
    requiresLabel,
    uploadMutation,
    algorithms,
    loadingAlgorithms,
    selectedAlgorithm,
    availableExports,
    hyperparameterFields,
    handleTaskChange,
    handleAlgorithmChange,
    setHyperparameter,
    labelledRows,
    unlabelledRows,
    createJobMutation,
    handleUpload,
    toggleFeature,
    validation,
    issues,
    restored,
    notice,
    setNotice,
    startOver,
  } = useNewJobWizard();

  const [summaryOpen, setSummaryOpen] = useState(false);
  const [announcement, setAnnouncement] = useState('');

  /** Move the focus to the control an issue belongs to. */
  const focusIssue = useCallback(
    (issue: ValidationIssue) => {
      const target = targetFor(issue);
      if (!target) return;
      if (issue.step !== step) {
        setStep(issue.step);
        window.setTimeout(() => {
          document.getElementById(target)?.focus();
        }, 0);
        return;
      }
      document.getElementById(target)?.focus();
    },
    [setStep, step],
  );

  /**
   * The behaviour of the primary action on an invalid step: expand the
   * summary, focus the first invalid field, and announce the count.
   */
  const explainIssues = useCallback(
    (stepIssues: ValidationIssue[]) => {
      setSummaryOpen(true);
      setAnnouncement(
        `${stepIssues.length} ${stepIssues.length === 1 ? 'issue' : 'issues'} to fix before continuing.`,
      );
      const first = stepIssues[0];
      if (first) focusIssue(first);
    },
    [focusIssue],
  );

  const handleContinue = (target: WizardStep) => {
    const stepIssues = issues.filter((issue) => issue.step === step);
    if (stepIssues.length > 0) {
      explainIssues(stepIssues);
      return;
    }
    setSummaryOpen(false);
    setAnnouncement('');
    setStep(target);
  };

  const handleStart = () => {
    const stepIssues = issues.filter((issue) => issue.step === 'model');
    if (stepIssues.length > 0) {
      explainIssues(stepIssues);
      return;
    }
    setSummaryOpen(false);
    setAnnouncement('');
    createJobMutation.mutate();
  };

  const showIssues = summaryOpen;

  // --- keyboard navigation (FEATURE-18) --------------------------------------
  // The wizard is one page with three steps, so a keyboard user should be able
  // to walk between them without reaching for the mouse: Alt+← / Alt+→ move to
  // the neighbouring step, and Alt+1..3 jump straight to one. Moving forward
  // still respects validation (`handleContinue`); moving back never blocks.
  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (!event.altKey) return;

      if (event.key === 'ArrowRight' || event.key === 'ArrowLeft') {
        const target = event.key === 'ArrowRight' ? nextStep(step) : previousStep(step);
        if (!target) return;
        event.preventDefault();
        handleContinue(target);
        return;
      }

      const digit = stepForDigit(event.key);
      if (digit) {
        event.preventDefault();
        handleContinue(digit);
      }
    };

    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  });
  // Deliberately no dependency array: the handler closes over `step` and the
  // issue list, and re-registering one listener per render is cheaper than
  // memoising the callback and keeping it in sync.

  // --- accidental-navigation guard (FEATURE-18) ------------------------------
  // A refresh or a tab close mid-wizard would discard the uploaded dataset, so
  // the browser is asked to confirm while a dataset is loaded and no job has
  // been created yet. In-app links are not guarded: the state is mirrored into
  // `sessionStorage`, so going back to the jobs list loses nothing.
  useEffect(() => {
    if (!dataset || createJobMutation.isSuccess) return;
    const onBeforeUnload = (event: BeforeUnloadEvent) => {
      event.preventDefault();
      // Chrome requires `returnValue` to be set to show the prompt.
      event.returnValue = '';
    };
    window.addEventListener('beforeunload', onBeforeUnload);
    return () => window.removeEventListener('beforeunload', onBeforeUnload);
  }, [createJobMutation.isSuccess, dataset]);

  return (
    <div className='mx-auto max-w-5xl px-4 py-8'>
      <header className='mb-6'>
        <h1 className='text-text-primary text-2xl font-semibold'>New job</h1>
        <p className='text-text-secondary mt-1 text-sm'>
          Upload a CSV, map columns, and start training.
        </p>
      </header>

      {restored && (
        <div className='alert alert-info mb-4' role='status'>
          <div className='flex flex-col gap-2'>
            <span className='text-sm'>Restored your unfinished job setup.</span>
            <div className='flex gap-2'>
              <button type='button' className='btn btn-sm' onClick={startOver}>
                Start over
              </button>
              <button
                type='button'
                className='btn btn-ghost btn-sm'
                onClick={() => setNotice(null)}>
                Dismiss
              </button>
            </div>
          </div>
        </div>
      )}

      {notice && (
        <div className='alert alert-warning mb-4' role='alert'>
          <span className='text-sm'>{notice}</span>
        </div>
      )}

      <WizardStepper step={step} validation={validation} />

      {issues.length > 0 && (
        <section className='mb-4' aria-label='Validation summary'>
          <button
            type='button'
            className='btn btn-sm btn-outline'
            aria-expanded={summaryOpen}
            aria-controls='wizard-validation-summary'
            onClick={() => setSummaryOpen((open) => !open)}>
            {summaryOpen ? 'Hide' : 'Fix'} {issues.length}{' '}
            {issues.length === 1 ? 'issue' : 'issues'} to continue
          </button>
          <p className='sr-only' role='status' aria-live='polite'>
            {announcement}
          </p>
          {summaryOpen && (
            <ul
              id='wizard-validation-summary'
              className='border-border bg-base-100 mt-2 flex flex-col gap-1 rounded-lg border p-3'>
              {issues.map((issue) => (
                <li key={`${issue.field}-${issue.message}`}>
                  <button
                    type='button'
                    className='link text-error text-sm'
                    onClick={() => focusIssue(issue)}>
                    {issue.message}
                  </button>
                </li>
              ))}
            </ul>
          )}
        </section>
      )}

      {step === 'dataset' && (
        <DatasetStep
          dataset={dataset}
          uploadError={uploadError}
          uploadMutation={uploadMutation}
          onUpload={handleUpload}
          onChangeFile={() => {
            setDataset(null);
            setFeatureColumns([]);
            setLabelColumn('');
            setStep('dataset');
          }}
          onContinue={() => handleContinue('mapping')}
          issues={validation.dataset}
          showIssues={showIssues}
        />
      )}

      {step === 'mapping' && (
        <MappingStep
          dataset={dataset}
          featureColumns={featureColumns}
          onToggleFeature={toggleFeature}
          taskType={taskType}
          onTaskChange={handleTaskChange}
          requiresLabel={requiresLabel}
          labelColumn={labelColumn}
          onLabelChange={onLabelChange}
          labelledRows={labelledRows}
          unlabelledRows={unlabelledRows}
          isReinforcement={isReinforcement}
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
          onBack={() => setStep('dataset')}
          onContinue={() => handleContinue('model')}
          issues={validation.mapping}
          showIssues={showIssues}
        />
      )}

      {step === 'model' && (
        <ModelStep
          dataset={dataset}
          taskType={taskType}
          algorithms={algorithms}
          loadingAlgorithms={loadingAlgorithms}
          algorithmId={algorithmId}
          handleAlgorithmChange={handleAlgorithmChange}
          selectedAlgorithm={selectedAlgorithm}
          hyperparameterFields={hyperparameterFields}
          hyperparameters={hyperparameters}
          setHyperparameter={setHyperparameter}
          contaminationMode={contaminationMode}
          setContaminationMode={setContaminationMode}
          availableExports={availableExports}
          exportState={exportState}
          setExport={setExport}
          createJobMutation={createJobMutation}
          onBack={() => setStep('mapping')}
          onStart={handleStart}
          issues={validation.model}
          showIssues={showIssues}
        />
      )}
    </div>
  );
}

const NewJob = () => (
  // A render crash mid-wizard falls back to the studio's alert rather than the
  // framework error page.
  <ErrorBoundary
    title='The job wizard could not be displayed'
    guidance='Reload the page or return to the jobs list.'>
    <NewJobWizard />
  </ErrorBoundary>
);

export default NewJob;
