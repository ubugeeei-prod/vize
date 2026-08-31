import type { ComputedRef, MaybeRefOrGetter } from "vue";

/** Serializable identity accepted by every collection primitive. */
export type CollectionKey = string | number;

/** How disabled items participate in focus navigation and typeahead. */
export type CollectionDisabledBehavior = "skip" | "focusable";

/** Direction understood by {@link CollectionRegistry.getNavigationKey}. */
export type CollectionNavigationDirection = "first" | "last" | "next" | "previous";

/** Why the registry changed its active key. */
export type CollectionActiveChangeReason =
  | "programmatic"
  | "navigation"
  | "typeahead"
  | "item-removed"
  | "item-disabled"
  | "registry-disposed";

/** Reactive sources registered for one logical collection item. */
export interface CollectionItemInput<Key extends CollectionKey, Value> {
  /** Stable, request-independent identity. Keys must be unique within a registry. */
  readonly key: Key;

  /** Consumer-owned data associated with this item. */
  readonly value: Value;

  /** Rendered element used for live DOM ordering and text extraction. */
  readonly element?: MaybeRefOrGetter<Element | null | undefined>;

  /**
   * Explicit typeahead text. `null` and `undefined` select accessible DOM text
   * extraction; an empty string deliberately opts the item out of typeahead.
   */
  readonly textValue?: MaybeRefOrGetter<string | null | undefined>;

  /** Whether activation is disabled. Navigation follows `disabledBehavior`. */
  readonly disabled?: MaybeRefOrGetter<boolean | undefined>;

  /**
   * Deterministic order for virtualized, portalled, or server-only items.
   *
   * If one item supplies an order, every item must supply a unique safe integer.
   * Otherwise DOM order is used once every item is connected in one document;
   * registration order is the SSR, partial-mount, and disconnected fallback.
   */
  readonly order?: MaybeRefOrGetter<number | undefined>;
}

/** Immutable, resolved view of a registered collection item. */
export interface CollectionItem<Key extends CollectionKey, Value> {
  /** Stable item identity. */
  readonly key: Key;

  /** Consumer-owned item data. */
  readonly value: Value;

  /** Current rendered element, or `null` during SSR and before mount. */
  readonly element: Element | null;

  /** Normalized text used by typeahead. */
  readonly textValue: string;

  /** Current disabled state. */
  readonly disabled: boolean;

  /** Explicit virtual order, or `undefined` when DOM order is authoritative. */
  readonly order: number | undefined;
}

/** Active-key transition reported to collection consumers. */
export interface CollectionActiveChange<Key extends CollectionKey> {
  /** New active key, or `null` when no navigable item remains. */
  readonly key: Key | null;

  /** Active key before the transition. */
  readonly previousKey: Key | null;

  /** Stable cause suitable for focus and announcement policies. */
  readonly reason: CollectionActiveChangeReason;
}

/** Options for {@link createCollectionRegistry}. */
export interface CollectionRegistryOptions<Key extends CollectionKey> {
  /**
   * Whether disabled items are omitted from navigation or remain focusable.
   *
   * Menus commonly use `"focusable"`; listboxes and roving tab stops commonly
   * use `"skip"`.
   *
   * @default "skip"
   */
  readonly disabledBehavior?: CollectionDisabledBehavior;

  /** Locale-aware comparator used by typeahead matching. */
  readonly collator?: Intl.Collator;

  /** Called synchronously after a distinct active-key transition. */
  readonly onActiveKeyChange?: (change: CollectionActiveChange<Key>) => void;
}

/** Options for directional collection navigation. */
export interface CollectionNavigationOptions<Key extends CollectionKey> {
  /**
   * Key navigation starts from.
   *
   * @default The registry active key
   */
  readonly fromKey?: Key | null;

  /**
   * Whether navigation wraps at the first and last item.
   *
   * @default false
   */
  readonly loop?: boolean;
}

/** Options for locale-aware collection text lookup. */
export interface CollectionTextSearchOptions<Key extends CollectionKey> {
  /**
   * Search begins after this key so repeated input cycles matching items.
   *
   * @default The registry active key
   */
  readonly fromKey?: Key | null;

  /**
   * Match normalized prefixes or the complete normalized text.
   *
   * @default "prefix"
   */
  readonly match?: "prefix" | "exact";

  /**
   * Whether search wraps to the beginning of the collection.
   *
   * @default true
   */
  readonly loop?: boolean;

  /** Per-search locale comparator override. */
  readonly collator?: Intl.Collator;
}

/** Idempotent lifecycle handle returned by {@link CollectionRegistry.register}. */
export interface CollectionRegistration<Key extends CollectionKey> {
  /** Registered stable key. */
  readonly key: Key;

  /** Whether this exact registration still owns its key. */
  readonly registered: ComputedRef<boolean>;

  /** Remove this registration and report whether it was present. */
  readonly unregister: () => boolean;
}

/** Type-safe registry shared by compound collection components. */
export interface CollectionRegistry<Key extends CollectionKey, Value> {
  /** All items in deterministic virtual, DOM, or registration order. */
  readonly items: ComputedRef<readonly CollectionItem<Key, Value>[]>;

  /** Items eligible for focus navigation under the disabled policy. */
  readonly navigableItems: ComputedRef<readonly CollectionItem<Key, Value>[]>;

  /** Logical focus key used by roving-tabindex and `aria-activedescendant` adapters. */
  readonly activeKey: ComputedRef<Key | null>;

  /** Register one item. Vue scope disposal unregisters it automatically. */
  readonly register: (input: CollectionItemInput<Key, Value>) => CollectionRegistration<Key>;

  /** Resolve the latest immutable snapshot for a key. */
  readonly getItem: (key: Key) => CollectionItem<Key, Value> | undefined;

  /**
   * Re-resolve reactive sources, DOM order, and extracted text immediately.
   *
   * A scoped MutationObserver calls this automatically for connected items;
   * the explicit hook supports custom renderers and observer-free environments.
   */
  readonly refresh: () => void;

  /** Set a registered, navigable active key, or clear it with `null`. */
  readonly setActiveKey: (key: Key | null) => boolean;

  /** Resolve a directional target without changing active state. */
  readonly getNavigationKey: (
    direction: CollectionNavigationDirection,
    options?: CollectionNavigationOptions<Key>,
  ) => Key | null;

  /** Move active state directionally and return the resulting target. */
  readonly moveActive: (
    direction: CollectionNavigationDirection,
    options?: CollectionNavigationOptions<Key>,
  ) => Key | null;

  /** Find the next locale-aware text match without changing active state. */
  readonly findByTextValue: (
    query: string,
    options?: CollectionTextSearchOptions<Key>,
  ) => Key | null;

  /** Move active state to the next locale-aware text match. */
  readonly moveActiveByTextValue: (
    query: string,
    options?: CollectionTextSearchOptions<Key>,
  ) => Key | null;

  /** Stop observation, clear items, and reject future mutations. */
  readonly dispose: () => boolean;
}
