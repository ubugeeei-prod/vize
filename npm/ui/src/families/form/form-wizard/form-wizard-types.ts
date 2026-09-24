/** Direction of a step transition. */
export type FormWizardDirection = "back" | "forward";

/** State published through the FormWizard `data-state` contract. */
export type FormWizardState = "complete" | "in-progress" | "validating";

/** Serializable progress persisted by a draft store. */
export interface FormWizardSnapshot<StepId extends string> {
  /** Step that was current. */
  readonly step: StepId;

  /** Steps the user has reached, in step order. */
  readonly visited: readonly StepId[];
}

/**
 * Pluggable persistence for wizard progress (for example `localStorage`,
 * `sessionStorage`, IndexedDB, or `@vizejs/composable`'s `useStorage`).
 * Loaded once after mount so server and client markup match; saved after every
 * step change; cleared on completion.
 */
export interface FormWizardDraftStore<StepId extends string> {
  /** Read the saved snapshot; unknown steps are ignored. */
  readonly load: () => FormWizardSnapshot<string> | null | undefined;

  /** Persist the current snapshot. */
  readonly save: (snapshot: FormWizardSnapshot<StepId>) => void;

  /** Remove the saved snapshot after completion or reset. */
  readonly clear?: () => void;
}

/** Context passed to the step validation gate. */
export interface FormWizardValidationContext<StepId extends string> {
  /** Step being left. */
  readonly step: StepId;

  /** Step that would become current. */
  readonly target: StepId;

  /** Aborted when a newer transition starts or the wizard unmounts. */
  readonly signal: AbortSignal;
}

/**
 * Per-step gate run before moving forward (or completing). Return `false`
 * (or resolve to it) to stay on the step.
 */
export type FormWizardValidate<StepId extends string> = (
  context: FormWizardValidationContext<StepId>,
) => boolean | Promise<boolean>;

/** State exposed to FormWizard slots and instances. */
export interface FormWizardSlotState<StepId extends string> {
  /** Every step id, in order. */
  readonly steps: readonly StepId[];

  /** Current step. */
  readonly current: StepId;

  /** Zero-based index of the current step. */
  readonly index: number;

  /** Number of steps. */
  readonly count: number;

  /** Completed fraction from 0 to 1 (`index / (count - 1)`, 1 for a single step). */
  readonly progress: number;

  /** Steps reached so far, in step order. */
  readonly visited: readonly StepId[];

  /** Whether the current step is the first. */
  readonly isFirst: boolean;

  /** Whether the current step is the last. */
  readonly isLast: boolean;

  /** Whether a validation gate is running. */
  readonly validating: boolean;

  /** Whether the last step passed its gate and `complete` was emitted. */
  readonly completed: boolean;

  /** Stable state token. */
  readonly state: FormWizardState;
}

/** Public instance API of FormWizard. */
export interface FormWizardExpose<StepId extends string> extends FormWizardSlotState<StepId> {
  /** Validate the current step, then advance (or complete on the last step). */
  readonly next: () => Promise<boolean>;

  /** Go to the previous step without validation. */
  readonly back: () => boolean;

  /** Go to a step; moving forward validates every step in between. */
  readonly goTo: (step: StepId) => Promise<boolean>;

  /** Return to the first step, forget visited steps, and clear the draft. */
  readonly reset: () => void;
}
