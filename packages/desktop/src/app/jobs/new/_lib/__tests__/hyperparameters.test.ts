import { describe, expect, it } from 'vitest';
import {
  fieldInputSchema,
  fieldSchema,
  hyperparameterFormSchema,
  hyperparametersSchema,
  parseFieldValue,
  validateHyperparameters,
} from '../hyperparameters';
import type { HyperparameterField } from '../types';

const integerField: HyperparameterField = {
  key: 'max_iter',
  label: 'Maximum iterations',
  min: 1,
  max: 1000,
  step: 1,
};

const clusterField: HyperparameterField = {
  key: 'n_clusters',
  label: 'Number of clusters',
  min: 2,
  step: 1,
};

const floatField: HyperparameterField = {
  key: 'learning_rate',
  label: 'Learning rate',
  min: 0.0001,
  max: 1,
  step: 0.0001,
};

describe('fieldSchema', () => {
  it('accepts an integer inside the range', () => {
    expect(fieldSchema(integerField, 100).safeParse(10).success).toBe(true);
  });

  it('rejects a value below the minimum with the catalogue message', () => {
    const result = fieldSchema(integerField, 100).safeParse(0);

    expect(result.success).toBe(false);
    expect(result.error?.issues[0].message).toBe('Maximum iterations must be at least 1.');
  });

  it('rejects a value above the maximum with the catalogue message', () => {
    const result = fieldSchema(integerField, 100).safeParse(2000);

    expect(result.success).toBe(false);
    expect(result.error?.issues[0].message).toBe('Maximum iterations must be at most 1000.');
  });

  it('rejects a fraction for a whole-number field', () => {
    const result = fieldSchema(integerField, 100).safeParse(2.5);

    expect(result.success).toBe(false);
    expect(result.error?.issues[0].message).toBe('Maximum iterations must be a whole number.');
  });

  it('allows a fraction for a continuous field', () => {
    expect(fieldSchema(floatField, 100).safeParse(0.5).success).toBe(true);
  });

  it('bounds a cluster count by the row count, not the catalogue', () => {
    const result = fieldSchema(clusterField, 10).safeParse(20);

    expect(result.success).toBe(false);
    expect(result.error?.issues[0].message).toBe('Number of clusters must be at most 9.');
  });
});

describe('hyperparametersSchema', () => {
  it('accepts an empty object', () => {
    expect(hyperparametersSchema([integerField], 100).safeParse({}).success).toBe(true);
  });

  it('accepts a value inside the range', () => {
    expect(hyperparametersSchema([integerField], 100).safeParse({ max_iter: 10 }).success).toBe(
      true,
    );
  });

  it('accepts the auto literal', () => {
    expect(hyperparametersSchema([integerField], 100).safeParse({ max_iter: 'auto' }).success).toBe(
      true,
    );
  });

  it('rejects a value outside the range', () => {
    expect(hyperparametersSchema([integerField], 100).safeParse({ max_iter: 0 }).success).toBe(
      false,
    );
  });
});

describe('fieldInputSchema', () => {
  it('accepts a blank control and the auto literal', () => {
    expect(fieldInputSchema(integerField, 100).safeParse('').success).toBe(true);
    expect(fieldInputSchema(integerField, 100).safeParse('auto').success).toBe(true);
  });

  it('accepts a numeric string inside the range', () => {
    expect(fieldInputSchema(integerField, 100).safeParse('10').success).toBe(true);
  });

  it('rejects a numeric string outside the range', () => {
    const result = fieldInputSchema(integerField, 100).safeParse('0');

    expect(result.success).toBe(false);
    expect(result.error?.issues[0].message).toBe('Maximum iterations must be at least 1.');
  });
});

describe('hyperparameterFormSchema', () => {
  it('accepts a form with every control valid', () => {
    const result = hyperparameterFormSchema([integerField], 100).safeParse({
      values: { max_iter: '10' },
    });

    expect(result.success).toBe(true);
  });

  it('accepts an unset control', () => {
    const result = hyperparameterFormSchema([integerField], 100).safeParse({
      values: { max_iter: '' },
    });

    expect(result.success).toBe(true);
  });

  it('reports the field message on an invalid control', () => {
    const result = hyperparameterFormSchema([integerField], 100).safeParse({
      values: { max_iter: '0' },
    });

    expect(result.success).toBe(false);
    expect(result.error?.issues[0].message).toBe('Maximum iterations must be at least 1.');
  });
});

describe('parseFieldValue', () => {
  it('treats a blank string as unset', () => {
    expect(parseFieldValue('')).toBeUndefined();
    expect(parseFieldValue('   ')).toBeUndefined();
  });

  it('keeps the auto literal', () => {
    expect(parseFieldValue('auto')).toBe('auto');
    expect(parseFieldValue(' auto ')).toBe('auto');
  });

  it('parses a number', () => {
    expect(parseFieldValue('12')).toBe(12);
    expect(parseFieldValue('0.5')).toBe(0.5);
  });

  it('treats a non-numeric string as unset', () => {
    expect(parseFieldValue('many')).toBeUndefined();
  });
});

describe('validateHyperparameters', () => {
  it('returns no error for a valid set', () => {
    expect(validateHyperparameters([integerField], { max_iter: 5 }, 100)).toEqual([]);
  });

  it('ignores an unset field', () => {
    expect(validateHyperparameters([integerField], { max_iter: '' }, 100)).toEqual([]);
    expect(validateHyperparameters([integerField], {}, 100)).toEqual([]);
  });

  it('reports one error per offending field', () => {
    const errors = validateHyperparameters(
      [integerField, clusterField],
      { max_iter: 0, n_clusters: 1 },
      100,
    );

    expect(errors.map((error) => error.key)).toEqual(['max_iter', 'n_clusters']);
  });

  it('accepts the auto contamination', () => {
    const contamination: HyperparameterField = {
      key: 'contamination',
      label: 'Contamination',
      min: 0,
      max: 0.5,
      step: 0.01,
    };

    expect(validateHyperparameters([contamination], { contamination: 'auto' }, 100)).toEqual([]);
  });

  it('rejects a contamination above 0.5', () => {
    const contamination: HyperparameterField = {
      key: 'contamination',
      label: 'Contamination',
      min: 0,
      max: 0.5,
      step: 0.01,
    };

    const errors = validateHyperparameters([contamination], { contamination: 0.8 }, 100);

    expect(errors[0].message).toBe('Contamination must be `auto` or between 0 and 0.5.');
  });

  it('accepts a numeric contamination inside the range', () => {
    const contamination: HyperparameterField = {
      key: 'contamination',
      label: 'Contamination',
      min: 0,
      max: 0.5,
      step: 0.01,
    };

    expect(validateHyperparameters([contamination], { contamination: 0.1 }, 100)).toEqual([]);
  });
});
