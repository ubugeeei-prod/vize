/** Values accepted by the native `aria-invalid` attribute. */
export type EditableAriaInvalid = boolean | "grammar" | "spelling";

/** How the preview enters edit mode. */
export type EditableActivationMode = "click" | "dblclick" | "focus" | "none";

/** Which interactions submit the draft. */
export type EditableSubmitMode = "blur" | "both" | "enter" | "none";

/** What an EditableTrigger does when activated. */
export type EditableTriggerAction = "cancel" | "edit" | "submit";

/** State published through the Editable `data-state` contract. */
export type EditableState = "disabled" | "editing" | "preview" | "readonly";

/** State exposed to Editable slots and instances. */
export interface EditableSlotState {
  /** Committed value. */
  readonly value: string;

  /** Text in the input while editing (equals `value` otherwise). */
  readonly draft: string;

  /** Whether the input is shown. */
  readonly editing: boolean;

  /** Whether the committed value is empty (the preview shows the placeholder). */
  readonly empty: boolean;

  /** Whether editing is disabled. */
  readonly disabled: boolean;

  /** Whether the value can be read but not edited. */
  readonly readOnly: boolean;

  /** Stable state token. */
  readonly state: EditableState;
}

/** Public instance API of Editable. */
export interface EditableExpose extends EditableSlotState {
  /** Rendered root element. */
  readonly root: HTMLDivElement | null;

  /** Enter edit mode (no-op while disabled or read-only); returns whether it started. */
  readonly edit: () => boolean;

  /** Commit the draft and leave edit mode; returns whether the value changed. */
  readonly submit: () => boolean;

  /** Discard the draft and leave edit mode. */
  readonly cancel: () => void;

  /** Replace the committed value; returns whether it changed. */
  readonly setValue: (value: string) => boolean;
}
