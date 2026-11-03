/**
 * Wizard keyboard navigation and the accidental-navigation guard (FEATURE-18).
 *
 * The wizard is a three-step form that keeps unsaved configuration in memory
 * (and mirrors it into `sessionStorage`). Two behaviours live here so the page
 * component stays declarative:
 *
 * - `nextStep` / `previousStep` — the ordered neighbours of a step, so the
 *   arrow keys can walk the wizard without hard-coding the order in the page.
 * - `isStepIndex` / `stepForDigit` — the mapping behind `Alt+1`/`Alt+2`/
 *   `Alt+3`, which jump to a step directly.
 */

import type { WizardStep } from './types';

/** The steps in the order they are presented. */
export const WIZARD_STEP_ORDER: WizardStep[] = ['dataset', 'mapping', 'model'];

/** The step after `step`, or `undefined` when `step` is the last one. */
export const nextStep = (step: WizardStep): WizardStep | undefined => {
  const index = WIZARD_STEP_ORDER.indexOf(step);
  return index >= 0 ? WIZARD_STEP_ORDER[index + 1] : undefined;
};

/** The step before `step`, or `undefined` when `step` is the first one. */
export const previousStep = (step: WizardStep): WizardStep | undefined => {
  const index = WIZARD_STEP_ORDER.indexOf(step);
  return index > 0 ? WIZARD_STEP_ORDER[index - 1] : undefined;
};

/**
 * The step a `Alt+<digit>` shortcut selects, or `undefined` for any other key.
 *
 * `Alt+1` is the first step, `Alt+3` the last; a digit with no step behind it
 * is ignored rather than clamped, so a typo does not silently jump somewhere
 * unexpected.
 */
export const stepForDigit = (digit: string): WizardStep | undefined => {
  const index = Number(digit) - 1;
  return Number.isInteger(index) && index >= 0 && index < WIZARD_STEP_ORDER.length
    ? WIZARD_STEP_ORDER[index]
    : undefined;
};
