import { StatusBadge } from '@/components/ui/StatusBadge';
import { type Job } from '../_lib/types';

interface JobDetailsProps {
  job: Job;
}

export const JobDetails = ({ job }: JobDetailsProps) => {
  return (
    <div className='mb-8 grid gap-6 sm:grid-cols-2 md:grid-cols-4'>
      <div className='card bg-base-100 p-6 shadow'>
        <p className='text-text-secondary text-sm font-medium'>Status</p>
        <p className='mt-1'>
          <StatusBadge status={job.status} />
        </p>
      </div>
      <div className='card bg-base-100 p-6 shadow'>
        <p className='text-text-secondary text-sm font-medium'>Task</p>
        <p className='mt-1'>
          <span className='badge badge-ghost badge-sm'>{job.taskType}</span>
        </p>
      </div>
      <div className='card bg-base-100 p-6 shadow'>
        <p className='text-text-secondary text-sm font-medium'>Algorithm</p>
        <p className='text-text-primary mt-1 font-mono text-sm'>{job.algorithmId}</p>
      </div>
      <div className='card bg-base-100 p-6 shadow'>
        <p className='text-text-secondary text-sm font-medium'>Created</p>
        <p className='text-text-primary mt-1 font-mono text-sm'>
          {new Date(job.createdAt).toLocaleString()}
        </p>
      </div>
    </div>
  );
};
