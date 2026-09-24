import type { SelectBy, SelectModelValue } from "../select/select-model.ts";
import type { ComboboxFilter } from "./combobox-filter.ts";

export type { ComboboxFilter } from "./combobox-filter.ts";

/** Open state mirrored to the Combobox data contract. */
export type ComboboxState = "closed" | "open";

/** Selection mode derived from the `multiple` prop. */
export type ComboboxSelectionMode = "multiple" | "single";

/**
 * APG autocomplete behavior, mirrored to `aria-autocomplete`.
 *
 * - `none`: the popup lists every option regardless of the text.
 * - `list`: the popup filters options by the typed text.
 * - `inline`: the input completes the first matching option inline; the list is not filtered.
 * - `both`: filtering plus inline completion.
 */
export type ComboboxAutocomplete = "both" | "inline" | "list" | "none";

/** Status of the async item loader. */
export type ComboboxLoadStatus = "error" | "idle" | "loading" | "success";

/** Context handed to `loadItems`. */
export interface ComboboxLoadContext {
  /** Aborted when a newer query supersedes this request or the combobox unmounts. */
  readonly signal: AbortSignal;
}

/** Async item source called with the current query. */
export type ComboboxLoader<T> = (
  query: string,
  context: ComboboxLoadContext,
) => Promise<readonly T[]> | readonly T[];

/** Values accepted by the native `aria-invalid` attribute. */
export type ComboboxAriaInvalid = boolean | "grammar" | "spelling";

/** Public props accepted by `ComboboxRoot`. */
export interface ComboboxRootProps<T, Multiple extends boolean = false> {
  /**
   * Consumer-owned base id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Controlled selection: `T | null`, or `readonly T[]` when `multiple` is `true`.
   *
   * @default undefined
   */
  readonly modelValue?: SelectModelValue<T, Multiple>;

  /**
   * Initial selection for uncontrolled use and the value restored by reset.
   *
   * @default undefined
   */
  readonly defaultValue?: SelectModelValue<T, Multiple>;

  /**
   * Allow selecting several options, rendered as chips. The literal type decides the model type.
   *
   * @default false
   */
  readonly multiple?: Multiple;

  /**
   * Every option value. Enables typed inference, root-side filtering exposed as `filteredItems`,
   * closed-state labels, and virtualization.
   *
   * @default undefined
   */
  readonly items?: readonly T[];

  /**
   * Async item source called (debounced) with the typed query. Its results replace `items`.
   *
   * @default undefined
   */
  readonly loadItems?: ComboboxLoader<T>;

  /**
   * Debounce in milliseconds before `loadItems` runs for a new query.
   *
   * @default 200
   */
  readonly debounce?: number;

  /**
   * Compare values by a property key or with a custom equality function.
   *
   * @default undefined
   */
  readonly by?: SelectBy<T>;

  /**
   * Human-readable text for a value: the input label after selection, chip text, and filter text.
   *
   * @default undefined
   */
  readonly itemText?: (value: T) => string;

  /**
   * Disable individual values.
   *
   * @default undefined
   */
  readonly itemDisabled?: (value: T) => boolean;

  /**
   * Serialize a value for native form submission.
   *
   * @default undefined
   */
  readonly formValue?: (value: T) => string;

  /**
   * Filter deciding option visibility; `false` disables filtering (server-side or custom lists).
   * Defaults to an accent- and case-insensitive "contains" match.
   *
   * @default undefined
   */
  readonly filter?: ComboboxFilter<T> | false;

  /**
   * APG autocomplete behavior, mirrored to `aria-autocomplete`.
   *
   * @default "list"
   */
  readonly autocomplete?: ComboboxAutocomplete;

  /**
   * Restrict the value to listed options. When `false`, free text is kept and submitted.
   *
   * @default true
   */
  readonly strict?: boolean;

  /**
   * Build a value from free text, enabling the create option and Enter-to-create.
   *
   * @default undefined
   */
  readonly createOption?: (text: string) => T;

  /**
   * Controlled input text. `undefined` selects uncontrolled behavior.
   *
   * @default undefined
   */
  readonly inputValue?: string;

  /**
   * Initial input text for uncontrolled use.
   *
   * @default ""
   */
  readonly defaultInputValue?: string;

  /**
   * Controlled open state. `undefined` selects uncontrolled behavior.
   *
   * @default undefined
   */
  readonly open?: boolean;

  /**
   * Initial open state for uncontrolled use.
   *
   * @default false
   */
  readonly defaultOpen?: boolean;

  /**
   * Open the popup when the input gains focus.
   *
   * @default false
   */
  readonly openOnFocus?: boolean;

  /**
   * Highlight the first visible option while typing.
   *
   * @default true
   */
  readonly autoHighlight?: boolean;

  /**
   * Close after an option is chosen. `undefined` closes in single mode only.
   *
   * @default undefined
   */
  readonly closeOnSelect?: boolean;

  /**
   * Let Escape clear the text (and single selection) while the popup is closed.
   *
   * @default true
   */
  readonly clearOnEscape?: boolean;

  /**
   * Wrap arrow-key navigation at the first and last option.
   *
   * @default false
   */
  readonly loop?: boolean;

  /**
   * Disable the input, options, and form submission.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Keep the value visible and submittable but prevent editing.
   *
   * @default false
   */
  readonly readonly?: boolean;

  /**
   * Require a selection (or free text when not strict) for native validation.
   *
   * @default false
   */
  readonly required?: boolean;

  /**
   * Form control name used by the hidden inputs.
   *
   * @default undefined
   */
  readonly name?: string;

  /**
   * Id of the form owner when the combobox renders outside its `<form>`.
   *
   * @default undefined
   */
  readonly form?: string;

  /**
   * Reading direction published to parts.
   *
   * @default "ltr"
   */
  readonly dir?: "ltr" | "rtl";

  /**
   * Invalid state announced to assistive technology.
   *
   * @default false
   */
  readonly ariaInvalid?: ComboboxAriaInvalid;
}

/** State exposed to `ComboboxRoot` slots. */
export interface ComboboxSlotState<T> {
  /** Selected values in selection order. */
  readonly selected: readonly T[];

  /** Display text for each selected value. */
  readonly selectedText: readonly string[];

  /** Current input text. */
  readonly inputValue: string;

  /** Text typed since the last commit, used for filtering and loading. */
  readonly query: string;

  /** `items` (or loaded items) after filtering; render these in items mode. */
  readonly filteredItems: readonly T[];

  /** Whether the popup is open. */
  readonly open: boolean;

  /** Whether no option is currently visible. */
  readonly empty: boolean;

  /** Whether `loadItems` is pending. */
  readonly loading: boolean;

  /** Async loader status. */
  readonly status: ComboboxLoadStatus;

  /** Last loader error, if any. */
  readonly error: unknown;

  /** Whether the combobox is disabled. */
  readonly disabled: boolean;

  /** Single or multiple selection. */
  readonly selectionMode: ComboboxSelectionMode;

  /** Stable state token. */
  readonly state: ComboboxState;

  /** Remove one selected value (chips). */
  readonly remove: (value: T) => boolean;
}

/** Public instance exposed by `ComboboxRoot`. */
export interface ComboboxRootExpose<T> {
  /** Selected values. */
  readonly selected: readonly T[];

  /** Current input text. */
  readonly inputValue: string;

  /** Whether the popup is open. */
  readonly open: boolean;

  /** Request a specific open state. */
  readonly setOpen: (open: boolean, event?: Event | null) => boolean;

  /** Replace the input text as if typed (filters and loads). */
  readonly setInputValue: (text: string) => void;

  /** Select a value (toggle in multiple mode). */
  readonly select: (value: T) => boolean;

  /** Deselect a value. */
  readonly deselect: (value: T) => boolean;

  /** Clear the selection and the text. */
  readonly clear: () => boolean;

  /** Restore defaults for selection and text. */
  readonly reset: () => void;

  /** Re-run the async loader for the current query. */
  readonly reload: () => void;

  /** Focus the input. */
  readonly focus: (options?: FocusOptions) => void;
}

/** State exposed to `ComboboxChip` slots. */
export interface ComboboxChipSlotState<T> {
  /** Chip value. */
  readonly value: T;

  /** Display text. */
  readonly text: string;

  /** Whether removal is disabled. */
  readonly disabled: boolean;

  /** Remove this value from the selection. */
  readonly remove: () => boolean;
}
