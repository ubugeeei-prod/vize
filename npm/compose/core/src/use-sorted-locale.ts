import { computed, toValue } from "vue";
import type { ComputedRef, MaybeRefOrGetter } from "vue";

import { useLocale } from "./locale.ts";

/** Options for {@link useSortedLocale}. */
export interface UseSortedLocaleOptions<Item> extends Intl.CollatorOptions {
  /**
   * Locale whose collation order is used. Pass it explicitly for
   * hydration-stable server rendering.
   *
   * @default the browser language, otherwise "en"
   */
  readonly locale?: string | Intl.Locale;

  /**
   * Sort from Z to A.
   *
   * @default false
   */
  readonly descending?: boolean;

  /**
   * Text to compare for each item. Required for non-string items.
   *
   * @default String(item)
   */
  readonly key?: (item: Item) => string;
}

/** Reactive locale-aware sorting returned by {@link useSortedLocale}. */
export interface SortedLocaleControls<Item> {
  /** Canonical locale in use. */
  readonly locale: ComputedRef<string>;

  /** Collator for the current locale and options. */
  readonly collator: ComputedRef<Intl.Collator>;

  /** New array sorted by locale collation; the source is never mutated. */
  readonly sorted: ComputedRef<readonly Item[]>;

  /**
   * Compare two strings with the current collator (ignores `descending`).
   *
   * @param left First string.
   * @param right Second string.
   * @returns Negative, zero, or positive.
   */
  readonly compare: (left: string, right: string) => number;
}

function browserLanguage(): string | undefined {
  return typeof window === "undefined" ? undefined : window.navigator.language;
}

/**
 * Sort strings locale-aware with `Intl.Collator`.
 *
 * @param items Reactive strings.
 * @param options Reactive locale, collation, and direction options.
 * @returns Reactive sorted copy and collation helpers.
 */
export function useSortedLocale(
  items: MaybeRefOrGetter<readonly string[]>,
  options?: MaybeRefOrGetter<UseSortedLocaleOptions<string>>,
): SortedLocaleControls<string>;

/**
 * Sort any items locale-aware by a string key with `Intl.Collator`.
 *
 * @param items Reactive items.
 * @param options Reactive locale, collation, direction, and required key.
 * @returns Reactive sorted copy and collation helpers.
 */
export function useSortedLocale<Item>(
  items: MaybeRefOrGetter<readonly Item[]>,
  options: MaybeRefOrGetter<
    UseSortedLocaleOptions<Item> & { readonly key: (item: Item) => string }
  >,
): SortedLocaleControls<Item>;

/**
 * Sort items with locale-aware collation (`Intl.Collator`): `"ä"` sorts
 * next to `"a"` in German but after `"z"` in Swedish, and `numeric: true`
 * orders `"item 10"` after `"item 9"`.
 *
 * The result is a new, stable-sorted array recomputed when the items,
 * options, or locale change; the source array is never mutated. Invalid
 * options surface as the platform `RangeError`. Derived state only: safe
 * and deterministic on the server when the locale is explicit.
 *
 * @example
 * ```ts
 * const { sorted } = useSortedLocale(users, { key: (user) => user.name, locale: "de", sensitivity: "base" });
 * ```
 *
 * @param items Reactive items.
 * @param options Reactive locale, collation, direction, and key.
 * @default options {}
 * @returns Reactive sorted copy and collation helpers.
 */
export function useSortedLocale<Item>(
  items: MaybeRefOrGetter<readonly Item[]>,
  options: MaybeRefOrGetter<UseSortedLocaleOptions<Item>> = {},
): SortedLocaleControls<Item> {
  const localeControls = useLocale(() => toValue(options).locale, { detect: browserLanguage });
  const collator = computed(() => {
    const {
      locale: _locale,
      descending: _descending,
      key: _key,
      ...collatorOptions
    } = toValue(options);
    return new Intl.Collator(localeControls.locale.value, collatorOptions);
  });
  const sorted = computed(() => {
    const { key = String, descending = false } = toValue(options);
    const active = collator.value;
    const direction = descending ? -1 : 1;
    const keyed = toValue(items).map((item) => ({ item, text: key(item) }));
    keyed.sort((left, right) => direction * active.compare(left.text, right.text));
    return keyed.map(({ item }) => item);
  });
  return {
    locale: localeControls.locale,
    collator,
    sorted,
    compare: (left, right) => collator.value.compare(left, right),
  };
}
