'use client';

import { useEffect, useMemo } from 'react';
import { Controller, useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { hyperparameterFormSchema } from '@/app/jobs/new/_lib/hyperparameters';
import type { HyperparameterField } from '@/app/jobs/new/_lib/types';
import type { ValidationIssue } from '@/app/jobs/new/_lib/validation';

/** The string-valued form shape; each control edits a string. */
interface FormValues {
  values: Record<string, string>;
}

export interface HyperparameterFormProps {
  /** The catalogue fields of the selected algorithm. */
  fields: HyperparameterField[];
  /** The dataset row count, which bounds cluster-style fields. */
  rowCount: number;
  /** The current values, owned by the wizard. */
  hyperparameters: Record<string, string | number | boolean>;
  /** Persist one value back to the wizard. */
  setHyperparameter: (key: string, value: string | number | boolean) => void;
  /** Live validation issues, used for the inline messages. */
  issues?: ValidationIssue[];
  /** True when the user asked to see the issues. */
  showIssues?: boolean;
}

/**
 * A schema-driven hyperparameter form (FEATURE-18).
 *
 * `react-hook-form` owns the field state and `zod` owns the rules; both are
 * derived from the same `HyperparameterField` metadata the API catalogue
 * returns, so adding a knob to the catalogue is enough to get the input, its
 * bounds, and its message.
 */
export const HyperparameterForm = ({
  fields,
  rowCount,
  hyperparameters,
  setHyperparameter,
  issues = [],
  showIssues = false,
}: HyperparameterFormProps) => {
  const defaultValues = useMemo<FormValues>(
    () => ({
      values: Object.fromEntries(
        fields.map((field) => [field.key, String(hyperparameters[field.key] ?? '')]),
      ),
    }),
    // Rebuilding on every keystroke would reset the form; the algorithm and
    // its field list are the real identity of this form.
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [fields],
  );

  const resolver = useMemo(
    () => zodResolver(hyperparameterFormSchema(fields, rowCount)),
    [fields, rowCount],
  );

  const {
    control,
    reset,
    formState: { errors },
  } = useForm<FormValues>({ defaultValues, resolver, mode: 'onChange' });

  // A new algorithm replaces the field list; reload the form with its values.
  useEffect(() => {
    reset(defaultValues);
  }, [defaultValues, reset]);

  return (
    <div className='grid gap-4 sm:grid-cols-2'>
      {fields.map((field) => {
        const issue = issues.find((entry) => entry.field === `hyperparameter-${field.key}`);
        const showIssue = showIssues && issue;
        const errorId = `hp-${field.key}-error`;
        const clientError = errors.values?.[field.key]?.message;
        const message = showIssue ? issue.message : clientError;
        const max = field.max ?? Math.max(2, rowCount - 1);

        return (
          <div key={field.key} className='form-control'>
            <label className='label' htmlFor={`hp-${field.key}`}>
              <span className='label-text'>{field.label}</span>
            </label>
            <Controller
              control={control}
              name={`values.${field.key}` as const}
              render={({ field: controlled }) => (
                <input
                  {...controlled}
                  id={`hp-${field.key}`}
                  type='number'
                  className='input input-bordered w-full'
                  min={field.min}
                  max={max}
                  step={field.step}
                  aria-invalid={message ? true : undefined}
                  aria-describedby={message ? errorId : undefined}
                  onChange={(event) => {
                    controlled.onChange(event);
                    setHyperparameter(
                      field.key,
                      event.target.value === '' ? '' : Number(event.target.value),
                    );
                  }}
                />
              )}
            />
            {field.help && <span className='text-text-tertiary mt-1 text-xs'>{field.help}</span>}
            {message && (
              <p id={errorId} className='text-error mt-1 text-xs'>
                {message}
              </p>
            )}
          </div>
        );
      })}
    </div>
  );
};
