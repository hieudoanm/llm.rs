import Link from 'next/link';
import { type Job } from '../_lib/types';

interface JobHeaderProps {
  job: Job;
}

export const JobHeader = ({ job }: JobHeaderProps) => {
  return (
    <div className='mb-6 flex items-center justify-between gap-4'>
      <h1 className='text-text-primary text-xl font-semibold tracking-tight'>
        Job <span className='text-text-tertiary font-mono text-sm'>{job.jobId}</span>
      </h1>
      <Link href='/jobs' className='btn btn-ghost'>
        Back to jobs
      </Link>
    </div>
  );
};
