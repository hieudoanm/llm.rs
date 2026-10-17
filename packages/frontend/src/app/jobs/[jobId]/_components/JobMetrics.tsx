import { metricInfo } from '@/lib/metrics';
import { PiDownload } from 'react-icons/pi';

interface JobMetricsProps {
  metrics: Record<string, number>;
  jobId: string;
}

export const JobMetrics = ({ metrics, jobId }: JobMetricsProps) => {
  const downloadMetrics = () => {
    const blob = new Blob([JSON.stringify(metrics, null, 2)], {
      type: 'application/json',
    });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `${jobId}-metrics.json`;
    a.click();
    URL.revokeObjectURL(url);
  };

  return (
    <section className='mb-8'>
      <div className='mb-4 flex items-center justify-between'>
        <h2 className='text-text-primary text-lg font-semibold'>Metrics</h2>
        <button
          className='btn btn-ghost btn-sm hover:bg-base-200 flex items-center gap-1.5'
          onClick={downloadMetrics}
          type='button'>
          <PiDownload className='icon-sm' aria-hidden='true' />
          <span>Download metrics.json</span>
        </button>
      </div>
      <dl className='border-border divide-border divide-y rounded-lg border'>
        {Object.entries(metrics).map(([key, value]) => {
          const { label, description } = metricInfo(key);
          return (
            <div key={key} className='flex items-start justify-between gap-4 px-4 py-3'>
              <dt className='text-text-secondary text-sm'>
                {label}
                {description && (
                  <span className='text-text-tertiary mt-0.5 block text-xs'>{description}</span>
                )}
              </dt>
              <dd className='text-text-primary font-mono text-sm'>
                {typeof value === 'number'
                  ? Number.isInteger(value)
                    ? String(value)
                    : value.toFixed(4)
                  : String(value)}
              </dd>
            </div>
          );
        })}
      </dl>
    </section>
  );
};
