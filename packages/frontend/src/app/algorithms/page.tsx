'use client';

import { useQuery } from '@tanstack/react-query';
import { ApiAlert } from '@/components/ui/ApiAlert';
import { apiGet, ApiError } from '@/lib/api';
import { QUERY_CACHE } from '@/lib/queryCache';
import { PiCpu, PiArrowRight } from 'react-icons/pi';
import type { IconType } from 'react-icons';

interface Algorithm {
  algorithmId: string;
  name: string;
  taskType: string;
  family: string;
  description: string;
  defaultHyperparameters: Record<string, unknown>;
  exports: string[];
}

interface AlgorithmsResponse {
  algorithms: Algorithm[];
}

interface AlgorithmGroup {
  taskType: string;
  title: string;
  blurb: string;
  icon: IconType;
  accent: string;
}

const GROUPS: AlgorithmGroup[] = [
  {
    taskType: 'classification',
    title: 'Classification',
    blurb: 'Predict a discrete class from the selected features.',
    icon: PiCpu,
    accent: 'text-primary',
  },
  {
    taskType: 'regression',
    title: 'Regression',
    blurb: 'Predict a numeric value.',
    icon: PiArrowRight,
    accent: 'text-accent',
  },
  {
    taskType: 'semi_supervised',
    title: 'Semi-supervised',
    blurb: 'Learn from labelled rows and refine with unlabelled ones.',
    icon: PiCpu,
    accent: 'text-primary',
  },
  {
    taskType: 'clustering',
    title: 'Clustering',
    blurb: 'Group similar rows without a label column.',
    icon: PiArrowRight,
    accent: 'text-accent',
  },
  {
    taskType: 'anomaly_detection',
    title: 'Anomaly detection',
    blurb: 'Flag rows that differ from the rest of the dataset.',
    icon: PiCpu,
    accent: 'text-primary',
  },
  {
    taskType: 'reinforcement_learning',
    title: 'Reinforcement learning',
    blurb: 'Fit a Q-function from logged transitions (offline, Fitted Q-Iteration).',
    icon: PiArrowRight,
    accent: 'text-accent',
  },
];

const AlgorithmCard = ({ algo }: { algo: Algorithm }) => (
  <div className='card bg-surface border-border rounded-xl border p-6 shadow-sm'>
    <div className='mb-3 flex items-start justify-between gap-4'>
      <div>
        <h3 className='text-text-primary mb-1 text-lg font-semibold'>{algo.name}</h3>
        <p className='text-text-tertiary font-mono text-sm'>{algo.algorithmId}</p>
      </div>
      <span className='badge badge-primary badge-sm'>{algo.taskType}</span>
    </div>
    <p className='text-text-secondary mb-4 text-sm leading-relaxed'>{algo.description}</p>
    <div className='mb-4 space-y-2'>
      <div>
        <span className='text-text-tertiary text-xs font-medium tracking-wide uppercase'>
          Exports
        </span>
        <div className='mt-1 flex flex-wrap gap-1.5'>
          {algo.exports.map((format) => (
            <span key={format} className='badge badge-ghost badge-sm font-mono'>
              .{format}
            </span>
          ))}
        </div>
      </div>
      {Object.keys(algo.defaultHyperparameters).length > 0 && (
        <div>
          <span className='text-text-tertiary text-xs font-medium tracking-wide uppercase'>
            Default Hyperparameters
          </span>
          <div className='mt-1 flex flex-wrap gap-1.5'>
            {Object.entries(algo.defaultHyperparameters).map(([key, value]) => (
              <span key={key} className='badge badge-outline badge-sm font-mono'>
                {key}={String(value)}
              </span>
            ))}
          </div>
        </div>
      )}
    </div>
    <p className='text-text-tertiary text-xs font-medium tracking-wide uppercase'>
      Family: <span className='font-mono normal-case'>{algo.family}</span>
    </p>
  </div>
);

const AlgorithmsPage = () => {
  const { data, isLoading, isError, error } = useQuery<AlgorithmsResponse, Error>({
    queryKey: ['algorithms'],
    queryFn: () => apiGet<AlgorithmsResponse>('/api/v1/algorithms'),
    ...QUERY_CACHE.catalogue,
  });

  if (isLoading) {
    return (
      <div className='mx-auto max-w-5xl px-4 pt-16 pb-12'>
        <h1 className='text-text-primary mb-8 text-2xl font-semibold tracking-tight'>Algorithms</h1>
        <div
          role='status'
          aria-label='Loading algorithms'
          className='flex items-center justify-center py-20'>
          <span className='loading loading-spinner loading-lg' aria-hidden='true' />
        </div>
      </div>
    );
  }

  if (isError) {
    return (
      <div className='mx-auto max-w-5xl px-4 pt-16 pb-12'>
        <h1 className='text-text-primary mb-8 text-2xl font-semibold tracking-tight'>Algorithms</h1>
        <ApiAlert
          variant='error'
          title='Unable to load algorithms'
          message={error instanceof ApiError ? error.message : 'Failed to fetch algorithms'}
        />
      </div>
    );
  }

  const algorithms = data?.algorithms ?? [];

  return (
    <div className='mx-auto max-w-5xl px-4 pt-16 pb-12'>
      <div className='mb-10'>
        <h1 className='text-text-primary mb-3 text-3xl font-bold tracking-tight sm:text-4xl'>
          Learning Algorithms
        </h1>
        <p className='text-text-secondary text-base leading-relaxed sm:text-lg'>
          All available algorithms in the skl catalogue. Filter by task type and explore parameters,
          exports, and descriptions.
        </p>
      </div>

      {GROUPS.map((group) => {
        const items = algorithms.filter((algo) => algo.taskType === group.taskType);
        if (items.length === 0) return null;
        const Icon = group.icon;
        return (
          <section key={group.taskType} className='mb-12'>
            <div className='mb-6 flex items-center gap-2'>
              <Icon className={`${group.accent} h-6 w-6`} aria-hidden='true' />
              <h2 className='text-text-primary text-2xl font-semibold tracking-tight'>
                {group.title}
              </h2>
              <span className='badge badge-outline'>{items.length}</span>
            </div>
            <p className='text-text-secondary mb-5 text-sm'>{group.blurb}</p>
            <div className='grid gap-6 md:grid-cols-2'>
              {items.map((algo) => (
                <AlgorithmCard key={algo.algorithmId} algo={algo} />
              ))}
            </div>
          </section>
        );
      })}
    </div>
  );
};

export default AlgorithmsPage;
