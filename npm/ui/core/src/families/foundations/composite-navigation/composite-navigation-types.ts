import type { ComputedRef, MaybeRefOrGetter } from "vue";

import type {
  CollectionItem,
  CollectionKey,
  CollectionRegistry,
} from "../collection/collection.ts";
import type {
  TypeaheadController,
  TypeaheadOptions,
} from "../../interaction/typeahead/typeahead.ts";

/** Physical axes recognized by composite arrow-key navigation. */
export type CompositeOrientation = "both" | "horizontal" | "vertical";

/** Reading direction used to map horizontal arrows. */
export type CompositeDirection = "ltr" | "rtl";

/** DOM focus-management strategy described by the WAI-ARIA APG. */
export type CompositeFocusStrategy = "active-descendant" | "roving";

/** Logical movement requested from a composite controller. */
export type CompositeNavigationIntent =
  | "first"
  | "focus"
  | "last"
  | "next"
  | "page-next"
  | "page-previous"
  | "pointer"
  | "previous"
  | "typeahead";

/** Imperative and keyboard movement commands accepted by `navigate`. */
export type CompositeNavigationCommand = Exclude<
  CompositeNavigationIntent,
  "focus" | "pointer" | "typeahead"
>;

/** Immutable snapshot published after logical active state changes. */
export interface CompositeNavigationChange<Key extends CollectionKey> {
  /** Newly active item. */
  readonly key: Key;

  /** Active item before navigation. */
  readonly previousKey: Key | null;

  /** Navigation operation that selected this item. */
  readonly intent: CompositeNavigationIntent;

  /** Native event responsible for navigation, or `null` for imperative calls. */
  readonly originalEvent: Event | null;

  /** Focus-management strategy used while synchronizing the DOM. */
  readonly focusStrategy: CompositeFocusStrategy;
}

/** Shared options for both composite focus-management strategies. */
export interface CompositeNavigationBaseOptions<Key extends CollectionKey, Value> {
  /** Ordered, mutation-aware logical item registry. */
  readonly registry: CollectionRegistry<Key, Value>;

  /**
   * Arrow-key axes recognized by this composite.
   *
   * @default "vertical"
   */
  readonly orientation?: MaybeRefOrGetter<CompositeOrientation | undefined>;

  /**
   * Reading direction used by horizontal arrows.
   *
   * @default "ltr"
   */
  readonly direction?: MaybeRefOrGetter<CompositeDirection | undefined>;

  /** Wrap arrow navigation at collection boundaries. */
  readonly loop?: MaybeRefOrGetter<boolean | undefined>;

  /** Suppress keyboard and pointer navigation while retaining active state. */
  readonly isDisabled?: MaybeRefOrGetter<boolean | undefined>;

  /** Number of logical items traversed by PageUp and PageDown. */
  readonly pageSize?: MaybeRefOrGetter<number | undefined>;

  /** Optional buffered typeahead integrated into the same active state. */
  readonly typeahead?: false | Omit<TypeaheadOptions<Key, Value>, "registry">;

  /** Called after the active item and DOM focus representation are synchronized. */
  readonly onNavigate?: (change: CompositeNavigationChange<Key>) => void;

  /**
   * Custom visibility policy for virtualized or scroll-managed collections.
   * The default calls `scrollIntoView({block: "nearest", inline: "nearest"})`
   * when the method exists and DOM focus did not already scroll the item.
   */
  readonly scrollIntoView?: (item: CollectionItem<Key, Value>, originalEvent: Event | null) => void;
}

/** Roving-tabindex options. */
export interface RovingNavigationOptions<
  Key extends CollectionKey,
  Value,
> extends CompositeNavigationBaseOptions<Key, Value> {
  /** @default "roving" */
  readonly focusStrategy?: "roving";

  /** Avoid browser scrolling when DOM focus moves; custom scrolling still runs. */
  readonly preventScroll?: MaybeRefOrGetter<boolean | undefined>;

  /** Optional stable ID projection for item props. */
  readonly getItemId?: (item: CollectionItem<Key, Value>) => string | undefined;
}

/** `aria-activedescendant` options with a required SSR-stable ID projection. */
export interface ActiveDescendantNavigationOptions<
  Key extends CollectionKey,
  Value,
> extends CompositeNavigationBaseOptions<Key, Value> {
  readonly focusStrategy: "active-descendant";

  /** Resolve the stable DOM ID referenced by `aria-activedescendant`. */
  readonly getItemId: (item: CollectionItem<Key, Value>) => string;
}

/** Closed, strategy-discriminated composite configuration. */
export type CompositeNavigationOptions<Key extends CollectionKey, Value> =
  | ActiveDescendantNavigationOptions<Key, Value>
  | RovingNavigationOptions<Key, Value>;

/** Dynamic props for the composite container. */
export interface CompositeContainerProps {
  readonly "aria-activedescendant"?: string;
  readonly tabindex?: 0;
  readonly onFocus: (event: FocusEvent) => void;
  readonly onKeydown: (event: KeyboardEvent) => void;
}

/** Dynamic props for one registered composite item. */
export interface CompositeItemProps {
  readonly id?: string;
  readonly tabindex?: -1 | 0;
  readonly onFocus: (event: FocusEvent) => void;
  readonly onPointerdown: (event: PointerEvent) => void;
}

/** Focus and keyboard adapter layered over a collection registry. */
export interface CompositeNavigationController<Key extends CollectionKey> {
  /** Registry-owned logical active key. */
  readonly activeKey: ComputedRef<Key | null>;

  /** Integrated typeahead controller, or `null` when typeahead is disabled. */
  readonly typeahead: TypeaheadController<Key> | null;

  /** Resolve current dynamic container attributes and stable handlers. */
  readonly getContainerProps: () => Readonly<CompositeContainerProps>;

  /** Resolve current dynamic item attributes and stable key-owned handlers. */
  readonly getItemProps: (key: Key) => Readonly<CompositeItemProps>;

  /** Move logical focus and synchronize the configured DOM representation. */
  readonly navigate: (
    intent: CompositeNavigationCommand,
    originalEvent?: Event | null,
  ) => Key | null;

  /** Release typeahead timers and cached handlers without disposing the registry. */
  readonly dispose: () => void;
}
