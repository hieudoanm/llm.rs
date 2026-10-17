import { cn } from '@/lib/cn';
import { type JobStatus } from '@/components/ui/StatusBadge';
import { STEPS } from '../_lib/constants';

interface JobStepperProps {
  status: JobStatus;
}

export const JobStepper = ({ status }: JobStepperProps) => {
  const currentStepIndex = STEPS.indexOf(status);
  const isFailed = status === 'failed';

  const stepClass = (index: number): string => {
    if (isFailed) return index === STEPS.length - 1 ? 'step-error' : '';
    if (index < currentStepIndex) return 'step-success';
    if (index === currentStepIndex) return 'step-primary';
    return '';
  };

  return (
    <div className='border-border bg-surface mb-8 w-full border-b'>
      <div className='mx-auto max-w-5xl px-4 py-6'>
        <ol className='steps horizontal w-full justify-between'>
          {STEPS.map((step, index) => (
            <li key={step} className={cn('step', stepClass(index))}>
              {step.charAt(0).toUpperCase() + step.slice(1)}
            </li>
          ))}
        </ol>
      </div>
    </div>
  );
};
