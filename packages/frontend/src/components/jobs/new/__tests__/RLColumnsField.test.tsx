import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { RLColumnsField } from '../RLColumnsField';
import { Dataset } from '@/app/jobs/new/_lib/types';

const dataset: Dataset = {
  datasetId: 'd1',
  sha256: 'abc',
  filename: 'experience.csv',
  byteSize: 100,
  columns: ['f1', 'f2', 'action', 'reward', 'nf1', 'nf2', 'done', 'episode'].map((name) => ({
    name,
    inferredDtype: 'float64',
    nullCount: 0,
    sampleValues: [1],
  })),
  rowCount: 50,
  createdAt: '2026-10-20T00:00:00Z',
};

const noop = () => undefined;

const setup = (overrides: Partial<React.ComponentProps<typeof RLColumnsField>> = {}) =>
  render(
    <RLColumnsField
      dataset={dataset}
      featureColumns={['f1', 'f2']}
      rlActionColumn=''
      setRlActionColumn={noop}
      rlRewardColumn=''
      setRlRewardColumn={noop}
      rlDoneColumn=''
      setRlDoneColumn={noop}
      rlEpisodeColumn=''
      setRlEpisodeColumn={noop}
      rlNextStateColumns={[]}
      toggleNextState={noop}
      {...overrides}
    />,
  );

describe('RLColumnsField', () => {
  it('renders the experience-column pickers (FEATURE-11)', () => {
    setup();
    expect(screen.getByLabelText('Action')).toBeInTheDocument();
    expect(screen.getByLabelText('Reward')).toBeInTheDocument();
    expect(screen.getByLabelText('Done (optional)')).toBeInTheDocument();
    expect(screen.getByLabelText('Episode (optional)')).toBeInTheDocument();
  });

  it('reports the selected action column', async () => {
    const setRlActionColumn = vi.fn();
    setup({ setRlActionColumn });
    await userEvent.selectOptions(screen.getByLabelText('Action'), 'action');
    expect(setRlActionColumn).toHaveBeenCalledWith('action');
  });

  it('shows the next-state hint with the state-feature count', () => {
    setup({ featureColumns: ['f1', 'f2'] });
    expect(screen.getByText(/one column per state feature \(2 selected\)/i)).toBeInTheDocument();
  });

  it('toggles a next-state column', async () => {
    const toggleNextState = vi.fn();
    setup({ toggleNextState });
    // The next-state chips are the only checkboxes in the field.
    await userEvent.click(screen.getAllByRole('checkbox')[4]);
    expect(toggleNextState).toHaveBeenCalledWith('nf1');
  });

  it('marks selected next-state columns', () => {
    setup({ rlNextStateColumns: ['nf1'] });
    expect(screen.getAllByRole('checkbox')[4]).toBeChecked();
  });
});
