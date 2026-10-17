import { describe, expect, it } from 'vitest';
import { ERROR_GUIDANCE, FALLBACK_GUIDANCE, errorDetails, guidanceFor } from '../errors';
import { ApiError } from '../api';

describe('guidanceFor', () => {
  it('prefers the backend code over the transport code', () => {
    const guidance = guidanceFor(
      new ApiError(
        'HTTP_ERROR',
        'Column "price" is missing',
        422,
        'req-1',
        undefined,
        'INSUFFICIENT_LABELS',
      ),
    );

    expect(guidance.title).toBe('Not enough labels');
    expect(guidance.retryable).toBe(false);
  });

  it('falls back to the transport code when the backend sent none', () => {
    const guidance = guidanceFor(new ApiError('NETWORK_ERROR', 'boom', 0));

    expect(guidance.title).toBe('Cannot reach the API');
    expect(guidance.action).toBe('Retry');
  });

  it('uses the generic guidance for an unknown code', () => {
    const guidance = guidanceFor(
      new ApiError('HTTP_ERROR', 'boom', 500, undefined, undefined, 'WAT_IS_THIS'),
    );

    expect(guidance).toEqual(FALLBACK_GUIDANCE);
  });

  it('returns the generic guidance without an error', () => {
    expect(guidanceFor(null)).toEqual(FALLBACK_GUIDANCE);
    expect(guidanceFor(undefined)).toEqual(FALLBACK_GUIDANCE);
  });

  it('interpolates the placeholders from the error details', () => {
    const guidance = guidanceFor(
      new ApiError('HTTP_ERROR', 'too large', 413, undefined, { limit: 25 }, 'UPLOAD_TOO_LARGE'),
    );

    expect(guidance.message).toContain('25 MiB');
  });

  it('interpolates the placeholders from the caller context', () => {
    const guidance = guidanceFor(new ApiError('REQUEST_TIMEOUT', 'slow', 0), { timeout: 2500 });

    expect(guidance.message).toContain('2500 ms');
  });

  it('gives every catalogue entry a title, a message and a retry flag', () => {
    for (const [code, entry] of Object.entries(ERROR_GUIDANCE)) {
      expect(entry.title, code).toBeTruthy();
      expect(entry.message, code).toBeTruthy();
      expect(typeof entry.retryable, code).toBe('boolean');
    }
  });

  it('marks the retryable codes as retryable', () => {
    expect(ERROR_GUIDANCE.NETWORK_ERROR.retryable).toBe(true);
    expect(ERROR_GUIDANCE.REQUEST_TIMEOUT.retryable).toBe(true);
    expect(ERROR_GUIDANCE.INTERNAL_ERROR.retryable).toBe(true);
    expect(ERROR_GUIDANCE.VALIDATION_ERROR.retryable).toBe(false);
  });

  it('offers an action for the codes the user can act on', () => {
    expect(ERROR_GUIDANCE.DATASET_NOT_FOUND.action).toBe('Upload again');
    expect(ERROR_GUIDANCE.JOB_NOT_FOUND.actionHref).toBe('/jobs');
    expect(ERROR_GUIDANCE.INVALID_COLUMN_MAP.actionHref).toBe('/jobs/new');
  });
});

describe('errorDetails', () => {
  it('returns nothing without an error', () => {
    expect(errorDetails(null)).toEqual([]);
  });

  it('reports the backend code, the status and the offending fields', () => {
    const details = errorDetails(
      new ApiError(
        'HTTP_ERROR',
        'boom',
        422,
        'req-1',
        {
          fields: [{ loc: ['body', 'name'], msg: 'field required' }],
        },
        'VALIDATION_ERROR',
      ),
    );

    expect(details).toContain('Backend code: VALIDATION_ERROR');
    expect(details).toContain('HTTP status: 422');
    expect(details).toContain('body.name: field required');
  });

  it('does not repeat the code when it matches the transport code', () => {
    const details = errorDetails(new ApiError('NETWORK_ERROR', 'boom', 0));

    expect(details.some((line) => line.startsWith('Backend code:'))).toBe(false);
  });
});
