'use client';

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
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  };

  const rows = Object.entries(metrics).map(([key, value]) => ({
    key,
    value,
  }));

  return (
    <section className='mb-8'>
      <h2 className='text-text-primary mb-4 text-lg font-semibold'>Metrics</h2>
      <div className='border-border overflow-x-auto rounded-lg border'>
        <table className='table-sm table'>
          <thead>
            <tr>
              <th>Metric</th>
              <th>Value</th>
            </tr>
          </thead>
          <tbody>
            {rows.map((row) => (
              <tr key={row.key}>
                <td className='font-mono text-sm'>{row.key}</td>
                <td className='text-sm'>{row.value.toFixed(4)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <button
        onClick={downloadMetrics}
        className='btn btn-outline mt-4'
        aria-label='Download metrics as JSON'>
        <PiDownload className='icon-sm' aria-hidden='true' />
        Download metrics
      </button>
    </section>
  );
};
