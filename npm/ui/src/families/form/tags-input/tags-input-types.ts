/** Reading direction used by tag arrow-key navigation. */
export type TagsInputDirection = "ltr" | "rtl";

/** Values accepted by the native `aria-invalid` attribute. */
export type TagsInputAriaInvalid = boolean | "grammar" | "spelling";

/** State exposed by the TagsInput root data contract. */
export type TagsInputState = "disabled" | "empty" | "filled" | "readonly";

/** State exposed by each TagsInputItem data contract. */
export type TagsInputItemState = "disabled" | "editing" | "idle";

/** Why a candidate tag was rejected. */
export type TagsInputInvalidReason = "duplicate" | "invalid" | "max" | "parse";

/** How a tag entered the collection. */
export type TagsInputAddSource = "api" | "blur" | "delimiter" | "enter" | "paste";

/** How a tag left the collection. */
export type TagsInputRemoveSource = "api" | "backspace" | "delete" | "delete-button";

/** Equality policy used for duplicate detection: a property name or a comparator. */
export type TagsInputBy<T> =
  | (T extends object ? keyof T & string : never)
  | ((left: T, right: T) => boolean);

/** Custom validator. `false` rejects with no message; a string rejects with that message. */
export type TagsInputValidator<T> = (tag: T, tags: readonly T[]) => boolean | string;

/** Immutable payload emitted when a candidate tag is rejected. */
export interface TagsInputInvalidEvent<T> {
  /** Stable rejection reason. */
  readonly reason: TagsInputInvalidReason;

  /** Validator-provided message, or `null` for built-in rejections. */
  readonly message: string | null;

  /** Trimmed candidate text as typed, pasted, or passed to `add`. */
  readonly text: string;

  /** Parsed tag, or `null` when parsing failed. */
  readonly tag: T | null;
}

/** State exposed to the TagsInputRoot default slot. */
export interface TagsInputSlotState<T> {
  /** Current tags in insertion order. */
  readonly tags: readonly T[];

  /** Current uncommitted input text. */
  readonly inputValue: string;

  /** Number of committed tags. */
  readonly count: number;

  /** Whether `max` has been reached. */
  readonly full: boolean;

  /** Whether every interaction and form submission is disabled. */
  readonly disabled: boolean;

  /** Whether tags are visible and submitted but cannot change. */
  readonly readonly: boolean;

  /** Whether the field is currently marked invalid. */
  readonly invalid: boolean;

  /** Stable state token for styling and tests. */
  readonly state: TagsInputState;
}

/** State exposed to TagsInputItem, TagsInputItemText, and TagsInputItemDelete slots. */
export interface TagsInputItemSlotState<T> {
  /** Tag value rendered by this item. */
  readonly value: T;

  /** Zero-based tag index. */
  readonly index: number;

  /** Display text resolved through `tagText`. */
  readonly text: string;

  /** Whether this tag currently owns DOM focus. */
  readonly active: boolean;

  /** Whether this tag is being edited inline. */
  readonly editing: boolean;

  /** Whether this tag cannot be focused, edited, or removed. */
  readonly disabled: boolean;

  /** Stable state token for styling and tests. */
  readonly state: TagsInputItemState;
}

/** Public instance exposed by TagsInputRoot. */
export interface TagsInputRootExpose<T> {
  /** Rendered root element. */
  readonly element: HTMLDivElement | null;

  /** Id used by the text input. */
  readonly id: string;

  /** Current tags. */
  readonly tags: readonly T[];

  /** Current uncommitted input text. */
  readonly inputValue: string;

  /** Stable root state token. */
  readonly state: TagsInputState;

  /** Parse, validate, and append one tag from text. Reports whether it was added. */
  readonly add: (text: string) => boolean;

  /** Validate and append one already-parsed tag. Reports whether it was added. */
  readonly addTag: (tag: T) => boolean;

  /** Remove the tag at `index`. Reports whether it was removed. */
  readonly remove: (index: number) => boolean;

  /** Remove every tag. Reports whether the value changed. */
  readonly clear: () => boolean;

  /** Replace the uncommitted input text. */
  readonly setInputValue: (text: string) => void;

  /** Move DOM focus to the text input. */
  readonly focus: (options?: FocusOptions) => void;

  /** Restore `defaultValue` and clear the input text. Reports whether tags changed. */
  readonly reset: () => boolean;
}

/** Public instance exposed by TagsInputItem. */
export interface TagsInputItemExpose {
  /** Rendered tag element. */
  readonly element: HTMLSpanElement | null;

  /** Move DOM focus to this tag. */
  readonly focus: (options?: FocusOptions) => void;

  /** Remove this tag. Reports whether it was removed. */
  readonly remove: () => boolean;

  /** Enter inline editing when the root is editable. Reports whether editing started. */
  readonly edit: () => boolean;
}

/** Public instance exposed by TagsInputInput. */
export interface TagsInputInputExpose {
  /** Rendered native text input. */
  readonly element: HTMLInputElement | null;

  /** Move DOM focus to the input. */
  readonly focus: (options?: FocusOptions) => void;
}
