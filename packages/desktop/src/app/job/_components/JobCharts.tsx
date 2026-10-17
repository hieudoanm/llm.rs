'use client';

interface JobChartsProps {
  jobId: string;
}

export const JobCharts = ({ jobId }: JobChartsProps) => {
  return (
    <section className='mb-8'>
      <h2 className='text-text-primary mb-4 text-lg font-semibold'>Charts</h2>
      <div className='card bg-base-100 p-6 text-center shadow'>
        <p className='text-text-secondary'>Charts for job {jobId}</p>
      </div>
    </section>
  );
};
