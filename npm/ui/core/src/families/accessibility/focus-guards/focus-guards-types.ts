import type { CSSProperties, MaybeRefOrGetter, ShallowRef } from "vue";

/** Logical sentinel position around a guarded focus region. */
export type FocusGuardPosition = "after" | "before";

/** Direction in which a sentinel redirects sequential focus. */
export type FocusGuardDirection = "backward" | "forward";

/** Why focus reached a sentinel. */
export type FocusGuardReason = "enter" | "wrap";

/** Preventable, immutable notification emitted before a guard redirects focus. */
export interface FocusGuardRedirectEvent {
  readonly type: "focus-guard-redirect";
  readonly position: FocusGuardPosition;
  readonly direction: FocusGuardDirection;
  readonly reason: FocusGuardReason;
  readonly target: HTMLElement | null;
  readonly relatedTarget: Element | null;
  readonly originalEvent: globalThis.FocusEvent;
  readonly defaultPrevented: boolean;
  readonly preventDefault: () => void;
}

/** Native props to merge onto one consumer-rendered sentinel. */
export interface FocusGuardProps {
  readonly "data-vize-focus-guard": FocusGuardPosition;
  readonly tabindex: number;
  readonly onFocus: (event: globalThis.FocusEvent) => void;
}

/** Reactive configuration for a pair of focus sentinels. */
export interface FocusGuardsOptions {
  /** Primary guarded region. It may be `null` during SSR and before mount. */
  readonly root: MaybeRefOrGetter<Element | null | undefined>;

  /** Portalled regions that share the primary region's focus order. */
  readonly branches?: MaybeRefOrGetter<readonly Element[] | null | undefined>;

  /** Whether an activated pair participates in its document's guard stack. @default true */
  readonly enabled?: MaybeRefOrGetter<boolean | undefined>;

  /** Avoid scrolling when a sentinel redirects focus. @default true */
  readonly preventScroll?: MaybeRefOrGetter<boolean | undefined>;

  /** Additional filter applied after native sequential-focus checks. */
  readonly accept?: (element: HTMLElement) => boolean;

  /** Fallback used when no sequentially focusable descendant remains. */
  readonly fallbackFocus?: () => HTMLElement | null | undefined;

  /**
   * Called before focus is redirected. If `preventDefault` retains sentinel focus, the consumer
   * must move focus elsewhere or provide a visible focus indicator.
   */
  readonly onRedirect?: (event: FocusGuardRedirectEvent) => void;
}

/** Lifecycle, state, and render props for one focus-guard pair. */
export interface FocusGuardsController {
  readonly isActive: Readonly<ShallowRef<boolean>>;
  readonly isGuarding: Readonly<ShallowRef<boolean>>;
  readonly beforeProps: Readonly<FocusGuardProps>;
  readonly afterProps: Readonly<FocusGuardProps>;
  readonly activate: () => void;
  readonly deactivate: () => void;
  readonly refresh: () => void;
  readonly dispose: () => void;
}

/** Optional zero-CSS visual preset for consumer-rendered sentinels. */
export type FocusGuardStylePreset = Readonly<CSSProperties>;
