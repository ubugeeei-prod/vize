import type {
  ComboboxFilter,
  ComboboxLoader,
  ComboboxSlotState,
} from "../combobox/combobox-types.ts";
import type { SelectBy } from "../select/select-model.ts";
import type { AutocompleteHistoryStorage } from "./autocomplete-history.ts";

export type { AutocompleteHistoryStorage } from "./autocomplete-history.ts";

/** Public props accepted by `AutocompleteRoot`. */
export interface AutocompleteRootProps<T> {
  /**
   * Consumer-owned base id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Controlled chosen suggestion. `undefined` selects uncontrolled behavior.
   *
   * @default undefined
   */
  readonly modelValue?: T | null;

  /**
   * Initial chosen suggestion for uncontrolled use.
   *
   * @default undefined
   */
  readonly defaultValue?: T | null;

  /**
   * Static suggestions filtered by the typed text.
   *
   * @default undefined
   */
  readonly items?: readonly T[];

  /**
   * Async suggestion source (address lookup, search API) called with the query.
   *
   * @default undefined
   */
  readonly loadItems?: ComboboxLoader<T>;

  /**
   * Debounce in milliseconds before `loadItems` runs.
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
   * Human-readable text for a suggestion.
   *
   * @default undefined
   */
  readonly itemText?: (value: T) => string;

  /**
   * Suggestion filter; `false` shows suggestions as returned by `loadItems`.
   *
   * @default undefined
   */
  readonly filter?: ComboboxFilter<T> | false;

  /**
   * Controlled input text.
   *
   * @default undefined
   */
  readonly inputValue?: string;

  /**
   * Controlled recent history, newest first. `undefined` selects uncontrolled behavior.
   *
   * @default undefined
   */
  readonly history?: readonly T[];

  /**
   * Initial recent history for uncontrolled use.
   *
   * @default []
   */
  readonly defaultHistory?: readonly T[];

  /**
   * Maximum number of remembered entries.
   *
   * @default 5
   */
  readonly maxHistory?: number;

  /**
   * Persistence adapter read after mount and written on every change.
   *
   * @default undefined
   */
  readonly historyStorage?: AutocompleteHistoryStorage<T>;

  /**
   * Turn submitted free text into a history entry (search-style). Omit to record chosen suggestions only.
   *
   * @default undefined
   */
  readonly fromText?: (text: string) => T;

  /**
   * Open the popup (showing history) when the input gains focus.
   *
   * @default true
   */
  readonly openOnFocus?: boolean;

  /**
   * Disable the input and suggestions.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Form field name; submits the typed text.
   *
   * @default undefined
   */
  readonly name?: string;
}

/** State exposed to the `AutocompleteRoot` slot. */
export interface AutocompleteSlotState<T> extends ComboboxSlotState<T> {
  /** Recent history, newest first. */
  readonly history: readonly T[];

  /** Whether the popup shows history because the query is empty. */
  readonly showingHistory: boolean;

  /** What to render: history while the query is empty, otherwise `filteredItems`. */
  readonly suggestions: readonly T[];

  /** Remove one history entry. */
  readonly forget: (value: T) => void;

  /** Clear the history, e.g. from a consumer "Clear recent" button. */
  readonly clearHistory: () => void;
}

/** Public instance exposed by `AutocompleteRoot`. */
export interface AutocompleteRootExpose<T> {
  /** Recent history, newest first. */
  readonly history: readonly T[];

  /** Add an entry to the history. */
  readonly remember: (value: T) => void;

  /** Remove an entry from the history. */
  readonly forget: (value: T) => void;

  /** Clear the history. */
  readonly clearHistory: () => void;
}
