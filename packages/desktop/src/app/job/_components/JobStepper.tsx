'use client';

import { type JobStatus } from '../_lib/constants';

interface JobStepperProps {
  status: JobStatus;
}

export const JobStepper = ({ status }: JobStepperProps) => {
  return (
    <div className='mb-4 flex items-center gap-2'>
      <span className='badge badge-ghost badge-sm'>{status}</span>
    </div>
  );
};
