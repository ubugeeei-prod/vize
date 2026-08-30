import type { MaybeRefOrGetter, ShallowRef } from "vue";

import type { CollectionKey, CollectionRegistry } from "../../../collection.ts";

/** Immutable snapshot emitted when buffered text matches a collection item. */
export interface TypeaheadMatch<Key extends CollectionKey> {
  /** Item selected by the locale-aware collection search. */
  readonly key: Key;

  /** Active item before the match. */
  readonly previousKey: Key | null;

  /** Normalized buffered query used for this search. */
  readonly query: string;

  /** Native key event responsible for the match, or `null` for manual input. */
  readonly originalEvent: KeyboardEvent | null;
}

/** Options for {@link createTypeahead}. */
export interface TypeaheadOptions<Key extends CollectionKey, Value> {
  /** Mutation-aware collection that owns active state and locale matching. */
  readonly registry: CollectionRegistry<Key, Value>;

  /**
   * Idle time before a new grapheme starts a fresh query.
   *
   * @default 500
   */
  readonly timeout?: MaybeRefOrGetter<number | undefined>;

  /**
   * Ignore input and synchronously clear a pending query while true.
   *
   * @default false
   */
  readonly isDisabled?: MaybeRefOrGetter<boolean | undefined>;

  /**
   * Treat Space as the first grapheme of a query. A space is always accepted
   * after another character so multi-word labels remain searchable.
   *
   * @default false
   */
  readonly allowSpace?: boolean;

  /** Comparator used to detect repeated graphemes before cycling matches. */
  readonly collator?: Intl.Collator;

  /** Called after active collection state moves to a text match. */
  readonly onMatch?: (match: TypeaheadMatch<Key>) => void;
}

/** Stable keyboard handler to merge onto a composite focus owner. */
export interface TypeaheadProps {
  readonly onKeydown: (event: KeyboardEvent) => void;
}

/** Buffered, locale-aware collection typeahead controller. */
export interface TypeaheadController<Key extends CollectionKey> {
  /** Current query, cleared after the configured timeout. */
  readonly query: Readonly<ShallowRef<string>>;

  /** Stable keyboard handler for declarative consumers. */
  readonly typeaheadProps: Readonly<TypeaheadProps>;

  /**
   * Consume exactly one Unicode grapheme and return the matching item.
   * Repeating one grapheme cycles through matching items.
   */
  readonly input: (grapheme: string, originalEvent?: KeyboardEvent | null) => Key | null;

  /** Clear buffered input and report whether any text was pending. */
  readonly reset: () => boolean;

  /** Clear timers and make imperative operations terminal. Safe to repeat. */
  readonly dispose: () => void;
}
