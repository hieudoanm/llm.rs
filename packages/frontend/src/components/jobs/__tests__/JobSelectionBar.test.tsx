import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { JobSelectionBar, type SelectedJob } from '../JobSelectionBar';

const jobs: SelectedJob[] = [
  { jobId: 'j1', name: 'baseline', status: 'completed', algorithmId: 'logistic_regression' },
  { jobId: 'j2', name: null, status: 'queued', algorithmId: 'random_forest' },
];

describe('JobSelectionBar', () => {
  it('renders nothing when no job is selected', () => {
    const { container } = render(
      <JobSelectionBar jobs={[]} onRemove={vi.fn()} onClear={vi.fn()} />,
    );

    expect(container).toBeEmptyDOMElement();
  });

  it('shows the selected jobs and the counter', () => {
    render(<JobSelectionBar jobs={jobs} onRemove={vi.fn()} onClear={vi.fn()} />);

    expect(screen.getByRole('region', { name: 'Selected jobs' })).toBeInTheDocument();
    expect(screen.getByText('2 of 5 selected')).toBeInTheDocument();
    expect(screen.getByText('baseline')).toBeInTheDocument();
  });

  it('removes one job', async () => {
    const onRemove = vi.fn();
    render(<JobSelectionBar jobs={jobs} onRemove={onRemove} onClear={vi.fn()} />);

    await userEvent.click(
      screen.getByRole('button', { name: 'Remove baseline from the selection' }),
    );

    expect(onRemove).toHaveBeenCalledWith('j1');
  });

  it('clears the whole selection', async () => {
    const onClear = vi.fn();
    render(<JobSelectionBar jobs={jobs} onRemove={vi.fn()} onClear={onClear} />);

    await userEvent.click(screen.getByRole('button', { name: 'Clear' }));

    expect(onClear).toHaveBeenCalledTimes(1);
  });

  it('offers the comparison once two jobs are selected', () => {
    render(<JobSelectionBar jobs={jobs} onRemove={vi.fn()} onClear={vi.fn()} />);

    expect(screen.getByRole('link', { name: 'Compare' })).toHaveAttribute(
      'href',
      '/jobs/compare?ids=j1,j2',
    );
  });

  it('does not offer the comparison for a single job', () => {
    render(<JobSelectionBar jobs={[jobs[0]]} onRemove={vi.fn()} onClear={vi.fn()} />);

    expect(screen.queryByRole('link', { name: 'Compare' })).not.toBeInTheDocument();
  });

  it('offers the bulk download when a job has artefacts', () => {
    render(
      <JobSelectionBar
        jobs={jobs}
        onRemove={vi.fn()}
        onClear={vi.fn()}
        downloadHref='http://127.0.0.1:8000/api/v1/jobs/artefacts.zip?ids=j1,j2'
        hasArtefacts
      />,
    );

    expect(screen.getByRole('link', { name: /Download all \(2\)/ })).toHaveAttribute(
      'href',
      'http://127.0.0.1:8000/api/v1/jobs/artefacts.zip?ids=j1,j2',
    );
  });

  it('hides the bulk download without artefacts', () => {
    render(
      <JobSelectionBar
        jobs={jobs}
        onRemove={vi.fn()}
        onClear={vi.fn()}
        downloadHref='http://127.0.0.1:8000/api/v1/jobs/artefacts.zip?ids=j1,j2'
      />,
    );

    expect(screen.queryByRole('link', { name: /Download all/ })).not.toBeInTheDocument();
  });
});
