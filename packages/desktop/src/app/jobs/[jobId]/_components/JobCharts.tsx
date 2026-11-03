import { ModelReport } from '@/components/charts/ModelReport';
import { AnyCharts } from '@/components/charts/types';

interface JobChartsProps {
  charts: Record<string, unknown>;
  jobId: string;
}

export const JobCharts = ({ charts, jobId }: JobChartsProps) => {
  return (
    <section className='mb-8'>
      <h2 className='text-text-primary mb-4 text-lg font-semibold'>Model report</h2>
      <ModelReport charts={charts as unknown as AnyCharts} jobId={jobId} />
    </section>
  );
};
