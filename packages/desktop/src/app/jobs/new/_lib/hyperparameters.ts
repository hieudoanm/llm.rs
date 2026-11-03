/**
 * Schema-driven hyperparameter validation (FEATURE-18).
 *
 * The catalogue already describes every knob as `HyperparameterField`
 * (`key`, `min`, `max`, `step`). This module turns that metadata into a Zod
 * schema, so the form and the request body share one source of truth instead
 * of re-deriving the same bounds by hand in each component.
 */

import { z } from 'zod';
import type { HyperparameterField } from './types';

/** One parsed hyperparameter value. */
export type HyperparameterValue = number | 'auto';

/**
 * The Zod schema of one hyperparameter field.
 *
 * An empty string means "not set" and is dropped from the request, so it is
 * accepted here and filtered before submit.
 */
export const fieldSchema = (field: HyperparameterField, rowCount: number) => {
  // `n_clusters`/`n_components` are bounded by the dataset size, not by the
  // catalogue: a k cannot exceed `rowCount - 1`.
  const datasetBound =
    field.key === 'n_clusters' || field.key === 'n_components'
      ? Math.max(2, rowCount - 1)
      : undefined;
  const max = datasetBound ?? field.max;
  const integer = field.step === 1;

  const schema = z
    .number({ error: `${field.label} must be a number.` })
    .min(field.min, `${field.label} must be at least ${field.min}.`);

  const bounded =
    max === undefined ? schema : schema.max(max, `${field.label} must be at most ${max}.`);
  return integer ? bounded.int(`${field.label} must be a whole number.`) : bounded;
};

/**
 * The Zod schema of one *string-valued* form control.
 *
 * The inputs are `type="number"` but their DOM value is always a string, so
 * the same rule needs a string-facing twin: `""` is "not set", `"auto"` is
 * the literal, and anything else is coerced before it is checked.
 */
export const fieldInputSchema = (field: HyperparameterField, rowCount: number) => {
  const base = fieldSchema(field, rowCount);
  return z.string().superRefine((raw, ctx) => {
    const trimmed = raw.trim();
    if (trimmed === '' || trimmed === 'auto') return;
    const parsed = base.safeParse(Number(trimmed));
    if (!parsed.success) {
      for (const issue of parsed.error.issues) {
        ctx.addIssue({ code: 'custom', message: issue.message });
      }
    }
  });
};

/** The Zod schema of the whole hyperparameter set of an algorithm. */
export const hyperparametersSchema = (
  fields: HyperparameterField[],
  rowCount: number,
): z.ZodType<Record<string, HyperparameterValue>> => {
  const shape: Record<string, z.ZodType> = {};
  for (const field of fields) {
    // An unset field is dropped, so the schema only constrains a set value.
    shape[field.key] = z.union([fieldSchema(field, rowCount), z.literal('auto')]).optional();
  }
  return z.object(shape) as unknown as z.ZodType<Record<string, HyperparameterValue>>;
};

/**
 * The form-shaped schema: a `values` record of string controls.
 *
 * This is what `zodResolver` validates in `HyperparameterForm`, so the inline
 * messages come from the same rules that power the wizard's summary panel.
 */
export const hyperparameterFormSchema = (fields: HyperparameterField[], rowCount: number) => {
  const shape: Record<string, z.ZodString> = {};
  for (const field of fields) {
    shape[field.key] = fieldInputSchema(field, rowCount);
  }
  return z.object({ values: z.object(shape) });
};

/**
 * Parse a raw form value into the wire value.
 *
 * Empty stays empty (`undefined`) so it is omitted from the request body;
 * `"auto"` stays the literal string; everything else becomes a number.
 */
export const parseFieldValue = (raw: string): number | 'auto' | undefined => {
  const trimmed = raw.trim();
  if (trimmed === '') return undefined;
  if (trimmed === 'auto') return 'auto';
  const numeric = Number(trimmed);
  return Number.isFinite(numeric) ? numeric : undefined;
};

export interface FieldError {
  key: string;
  message: string;
}

/**
 * Validate every set hyperparameter against its schema.
 *
 * Returns one error per offending field, in catalogue order, so the summary
 * panel and the inline messages agree.
 */
export const validateHyperparameters = (
  fields: HyperparameterField[],
  values: Record<string, string | number | boolean>,
  rowCount: number,
): FieldError[] => {
  const errors: FieldError[] = [];
  for (const field of fields) {
    const raw = values[field.key];
    if (raw === undefined || raw === null || raw === '') continue;

    // Contamination is the one enum-valued knob.
    if (field.key === 'contamination') {
      if (raw === 'auto') continue;
      const numeric = Number(raw);
      if (!Number.isFinite(numeric) || numeric <= 0 || numeric > 0.5) {
        errors.push({
          key: field.key,
          message: 'Contamination must be `auto` or between 0 and 0.5.',
        });
      }
      continue;
    }

    const parsed = fieldSchema(field, rowCount).safeParse(Number(raw));
    if (!parsed.success) {
      const first = parsed.error.issues[0];
      errors.push({ key: field.key, message: first?.message ?? `${field.label} is invalid.` });
    }
  }
  return errors;
};
