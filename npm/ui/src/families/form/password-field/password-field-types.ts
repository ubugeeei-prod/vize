/** Values accepted by the native `aria-invalid` attribute. */
export type PasswordFieldAriaInvalid = boolean | "grammar" | "spelling";

/** State published through the PasswordField `data-state` contract. */
export type PasswordFieldState = "disabled" | "hidden" | "readonly" | "visible";

/** State exposed to the PasswordField slot and instance. */
export interface PasswordFieldSlotState<Strength> {
  /** Current password. */
  readonly value: string;

  /** Whether the password is shown as plain text. */
  readonly visible: boolean;

  /** Whether Caps Lock was on at the last key event inside the input. */
  readonly capsLock: boolean;

  /** Result of `evaluateStrength`, or `undefined` when no evaluator is supplied. */
  readonly strength: Strength | undefined;

  /** Whether editing is disabled. */
  readonly disabled: boolean;

  /** Whether edits are locked while focus remains available. */
  readonly readOnly: boolean;

  /** Stable state token. */
  readonly state: PasswordFieldState;
}

/** Public instance API of PasswordField. */
export interface PasswordFieldExpose<Strength> extends PasswordFieldSlotState<Strength> {
  /** Rendered root element. */
  readonly root: HTMLDivElement | null;

  /** Focus the password input. */
  readonly focus: (options?: FocusOptions) => void;

  /** Show or hide the password; returns whether visibility changed. */
  readonly setVisible: (visible: boolean) => boolean;

  /** Toggle visibility; returns the new visibility. */
  readonly toggleVisible: () => boolean;

  /** Replace the password; returns whether it changed. */
  readonly setValue: (value: string) => boolean;

  /** Restore the default password and visibility. */
  readonly reset: () => void;
}
