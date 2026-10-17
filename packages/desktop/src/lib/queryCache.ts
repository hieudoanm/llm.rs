import { keepPreviousData } from '@tanstack/react-query';

/**
 * Shared react-query tuning (FEATURE-18).
 *
 * Each endpoint has a different freshness budget, so the durations live here
 * rather than in the global defaults:
 *
 * - `CATALOGUE` — the algorithm catalogue only changes on deploy.
 * - `INSIGHTS` — a dataset profile is immutable once uploaded.
 * - `HISTORY` — the job list changes as jobs are submitted and finish.
 * - `COMPARISON` — a comparison is a pure function of its ids.
 * - `LIVE_JOB` — a single job is polled while it is non-terminal.
 */
export const QUERY_CACHE = {
  catalogue: { staleTime: 5 * 60_000, gcTime: 30 * 60_000 },
  insights: { staleTime: 5 * 60_000, gcTime: 30 * 60_000 },
  history: { staleTime: 10_000, gcTime: 5 * 60_000 },
  comparison: { staleTime: 30_000, gcTime: 5 * 60_000 },
  liveJob: { staleTime: 0, gcTime: 5 * 60_000 },
} as const;

/**
 * Keep the previous page's jobs visible while a new filter combination loads,
 * so switching filters never flashes an empty list.
 */
export const keepPrevious = { placeholderData: keepPreviousData } as const;
