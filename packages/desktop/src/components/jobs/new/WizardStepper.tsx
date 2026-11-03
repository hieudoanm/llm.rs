'use client';

import { cn } from '@/lib/cn';
import { WizardStep } from '@/app/jobs/new/_lib/types';
import { ValidationState, issuesForStep } from '@/app/jobs/new/_lib/validation';
import { PiCheckCircle, PiWarningCircle } from 'react-icons/pi';

type Props = {
  step: WizardStep;
  /** The live validation result of every step. */
  validation?: ValidationState;
};

const ORDER: WizardStep[] = ['dataset', 'mapping', 'model'];

const STEP_LABELS: Record<WizardStep, string> = {
  dataset: 'Dataset',
  mapping: 'Mapping',
  model: 'Model',
};

/**
 * The step indicator of the wizard.
 *
 * Every step shows one of three states — complete, current, or blocked — and
 * a blocked step carries the reason as its accessible name.
 */
export const WizardStepper = ({ step, validation }: Props) => {
  const currentIndex = ORDER.indexOf(step);

  return (
    <div className='border-border bg-surface mb-6 w-full border-b'>
      <div className='mx-auto max-w-5xl px-4 py-6'>
        <ol className='steps horizontal w-full justify-between'>
          {ORDER.map((entry, index) => {
            const issues = validation ? issuesForStep(validation, entry) : [];
            const isCurrent = entry === step;
            const isComplete = index < currentIndex && issues.length === 0;
            const isBlocked = !isCurrent && index > currentIndex && issues.length > 0;

            const state = isCurrent
              ? 'current'
              : isComplete
                ? 'complete'
                : isBlocked
                  ? 'blocked'
                  : 'upcoming';
            const reason = issues[0]?.message;

            return (
              <li
                key={entry}
                className={cn('step', (isCurrent || isComplete) && 'step-primary')}
                data-state={state}
                aria-current={isCurrent ? 'step' : undefined}
                aria-label={
                  reason
                    ? `${STEP_LABELS[entry]} — blocked: ${reason}`
                    : `${STEP_LABELS[entry]} — ${state}`
                }>
                <span className='flex items-center gap-1'>
                  {isComplete && <PiCheckCircle className='icon-xs' aria-hidden='true' />}
                  {isBlocked && <PiWarningCircle className='icon-xs' aria-hidden='true' />}
                  {STEP_LABELS[entry]}
                </span>
              </li>
            );
          })}
        </ol>
      </div>
    </div>
  );
};
