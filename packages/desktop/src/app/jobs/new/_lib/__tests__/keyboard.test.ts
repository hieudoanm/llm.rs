import { describe, expect, it } from 'vitest';
import { WIZARD_STEP_ORDER, nextStep, previousStep, stepForDigit } from '../keyboard';

describe('WIZARD_STEP_ORDER', () => {
  it('is the order the steps are presented in', () => {
    expect(WIZARD_STEP_ORDER).toEqual(['dataset', 'mapping', 'model']);
  });
});

describe('nextStep', () => {
  it('walks forward one step', () => {
    expect(nextStep('dataset')).toBe('mapping');
    expect(nextStep('mapping')).toBe('model');
  });

  it('stops at the last step', () => {
    expect(nextStep('model')).toBeUndefined();
  });
});

describe('previousStep', () => {
  it('walks back one step', () => {
    expect(previousStep('model')).toBe('mapping');
    expect(previousStep('mapping')).toBe('dataset');
  });

  it('stops at the first step', () => {
    expect(previousStep('dataset')).toBeUndefined();
  });
});

describe('stepForDigit', () => {
  it('maps 1..3 onto the steps', () => {
    expect(stepForDigit('1')).toBe('dataset');
    expect(stepForDigit('2')).toBe('mapping');
    expect(stepForDigit('3')).toBe('model');
  });

  it('ignores a digit with no step behind it', () => {
    expect(stepForDigit('0')).toBeUndefined();
    expect(stepForDigit('4')).toBeUndefined();
  });

  it('ignores a non-digit', () => {
    expect(stepForDigit('a')).toBeUndefined();
    expect(stepForDigit('')).toBeUndefined();
  });
});
