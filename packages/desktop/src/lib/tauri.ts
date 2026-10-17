/**
 * Tauri transport layer (FEATURE-18).
 *
 * Every call goes through this module. The wrapper uses `tauri::invoke()` to
 * call Rust commands and normalises every failure into a single `TauriError`
 * with a discriminated `code`.
 */

import { invoke } from '@tauri-apps/api/core';

/**
 * The transport-level error codes raised by the wrapper.
 *
 * The backend's own error codes survive in `TauriError.serverCode`, so a
 * backend error is never flattened into `INVOKE_ERROR`.
 */
export type TauriErrorCode = 'INVOKE_ERROR' | 'INVALID_RESPONSE' | 'UNKNOWN_ERROR';

export class TauriError extends Error {
  constructor(
    public code: TauriErrorCode,
    public message: string,
    public status: number,
    public requestId?: string,
    /** The backend's own error code, when the response carried an envelope. */
    public serverCode?: string,
    public details?: Record<string, unknown>,
  ) {
    super(message);
    this.name = 'TauriError';
  }

  /** True when the failure may be worth another attempt. */
  get retryable(): boolean {
    if (this.code === 'INVOKE_ERROR') return true;
    return false;
  }
}

/** Map a Tauri invoke error string into a TauriError. */
const toTauriError = (err: unknown): TauriError => {
  if (err instanceof TauriError) return err;

  const message = err instanceof Error ? err.message : String(err);

  // Try to parse a JSON error payload from the backend.
  let parsed: { code?: string; message?: string; requestId?: string; details?: unknown } | null;
  try {
    parsed = JSON.parse(message) as {
      code?: string;
      message?: string;
      requestId?: string;
      details?: unknown;
    };
  } catch {
    parsed = null;
  }

  if (parsed?.code) {
    return new TauriError(
      'INVOKE_ERROR',
      parsed.message ?? message,
      500,
      parsed.requestId,
      parsed.code,
      parsed.details as Record<string, unknown>,
    );
  }

  return new TauriError('INVOKE_ERROR', message, 500);
};

/**
 * Invoke a Tauri command and return the typed result.
 *
 * Throws `TauriError` on failure.
 */
export const tauriInvoke = async <T = unknown>(
  cmd: string,
  args?: Record<string, unknown>,
): Promise<T> => {
  try {
    return (await invoke<T>(cmd, args)) as T;
  } catch (err) {
    throw toTauriError(err);
  }
};

/** `GET` algorithms, optionally filtered by task type. */
export const tauriGetAlgorithms = async <T = unknown>(taskType?: string): Promise<T> =>
  tauriInvoke<T>('list_algorithms', { task: taskType ?? undefined });

/** `POST` a dataset from raw CSV bytes. */
export const tauriPostDataset = async <T = unknown>(
  bytes: Uint8Array,
  filename: string,
): Promise<T> => tauriInvoke<T>('upload_dataset_from_bytes', { bytes, filename });

/** `POST` a training job. */
export const tauriPostTrain = async <T = unknown>(request: Record<string, unknown>): Promise<T> =>
  tauriInvoke<T>('train_model', { request });

/** `GET` a single job. */
export const tauriGetJob = async <T = unknown>(jobId: string): Promise<T> =>
  tauriInvoke<T>('get_job', { job_id: jobId });

/** `GET` all jobs. */
export const tauriGetJobs = async <T = unknown>(): Promise<T> => tauriInvoke<T>('list_jobs', {});

/** `PATCH` job metadata. */
export const tauriPatchJob = async <T = unknown>(
  jobId: string,
  name: string | null,
  tags: string[],
): Promise<T> => tauriInvoke<T>('patch_job', { job_id: jobId, name, tags });

/** `GET` dataset insights. */
export const tauriGetDatasetInsights = async <T = unknown>(datasetId: string): Promise<T> =>
  tauriInvoke<T>('get_dataset_insights', { dataset_id: datasetId });

/** `GET` compare jobs by IDs. */
export const tauriGetCompareJobs = async <T = unknown>(ids: string[]): Promise<T> =>
  tauriInvoke<T>('compare_jobs', { ids: ids.join(',') });

/** `GET` artefacts for a job. */
export const tauriGetArtefacts = async <T = unknown>(jobId: string): Promise<T> =>
  tauriInvoke<T>('list_artefacts', { job_id: jobId });

/** `GET` download an artefact as base64. */
export const tauriDownloadArtefact = async (jobId: string, artefactId: string): Promise<string> =>
  tauriInvoke<string>('download_artefact', { job_id: jobId, artefact_id: artefactId });
