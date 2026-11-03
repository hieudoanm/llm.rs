import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { StatusBadge, type JobStatus } from '../StatusBadge';

const CASES: Array<[JobStatus, string, string]> = [
  ['queued', 'Waiting for worker', 'badge-warning'],
  ['validating', 'Checking CSV and mapping', 'badge-warning'],
  ['training', 'Fitting model', 'badge-warning'],
  ['evaluating', 'Scoring hold-out split', 'badge-warning'],
  ['exporting', 'Writing ONNX', 'badge-warning'],
  ['completed', 'Ready to download', 'badge-success'],
  ['failed', 'Failed', 'badge-error'],
];

describe('StatusBadge', () => {
  it.each(CASES)('renders %s with its label and badge colour', (status, label, badgeClass) => {
    render(<StatusBadge status={status} />);

    expect(screen.getByText(label)).toHaveClass('badge', badgeClass);
  });

  it('renders the raw status when labels are hidden', () => {
    render(<StatusBadge status='training' showLabel={false} />);

    expect(screen.getByText('training')).toBeInTheDocument();
    expect(screen.queryByText('Fitting model')).not.toBeInTheDocument();
  });

  it('merges a custom class name', () => {
    render(<StatusBadge status='completed' className='ml-2' />);

    expect(screen.getByText('Ready to download')).toHaveClass('badge-success', 'ml-2');
  });
});
