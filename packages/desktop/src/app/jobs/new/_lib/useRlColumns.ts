import { useState } from 'react';
import { RlColumns } from './types';

/**
 * Experience-column state for offline reinforcement-learning jobs (FEATURE-11).
 *
 * Tracks the action/reward/done/episode columns and the next-state columns, and
 * derives the `rlColumns` payload plus a readiness flag used to gate the step.
 */
export const useRlColumns = (featureColumns: string[]) => {
  const [actionColumn, setActionColumn] = useState<string>('');
  const [rewardColumn, setRewardColumn] = useState<string>('');
  const [doneColumn, setDoneColumn] = useState<string>('');
  const [episodeColumn, setEpisodeColumn] = useState<string>('');
  const [nextStateColumns, setNextStateColumns] = useState<string[]>([]);

  const toggleNextState = (name: string) => {
    setNextStateColumns((current) =>
      current.includes(name) ? current.filter((c) => c !== name) : [...current, name],
    );
  };

  const reset = () => {
    setActionColumn('');
    setRewardColumn('');
    setDoneColumn('');
    setEpisodeColumn('');
    setNextStateColumns([]);
  };

  const rlColumns: RlColumns = {
    actionColumn,
    rewardColumn,
    nextStateColumns,
    doneColumn: doneColumn || undefined,
    episodeColumn: episodeColumn || undefined,
  };

  // Next-state columns must mirror the state features one-for-one, and an
  // experience column may not double as a state feature (FR-111).
  const nextStateMatches =
    nextStateColumns.length === featureColumns.length && featureColumns.length > 0;
  const columnsDistinct = [
    actionColumn,
    rewardColumn,
    doneColumn,
    episodeColumn,
    ...nextStateColumns,
  ]
    .filter(Boolean)
    .every((col) => !featureColumns.includes(col));
  const hasTarget = nextStateMatches || Boolean(episodeColumn);
  const rlReady = Boolean(actionColumn) && Boolean(rewardColumn) && hasTarget && columnsDistinct;

  return {
    rlColumns,
    rlReady,
    rlActionColumn: actionColumn,
    setRlActionColumn: setActionColumn,
    rlRewardColumn: rewardColumn,
    setRlRewardColumn: setRewardColumn,
    rlDoneColumn: doneColumn,
    setRlDoneColumn: setDoneColumn,
    rlEpisodeColumn: episodeColumn,
    setRlEpisodeColumn: setEpisodeColumn,
    rlNextStateColumns: nextStateColumns,
    toggleNextState,
    resetRlColumns: reset,
  };
};
