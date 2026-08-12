import type { MaybeRefOrGetter, ShallowRef } from "vue";

/** Preventable lifecycle notification for automatic focus movement. */
export interface FocusScopeAutoFocusEvent {
  readonly type: "mount" | "unmount";
  readonly target: HTMLElement | null;
  readonly defaultPrevented: boolean;
  readonly preventDefault: () => void;
}

/** Programmatic movement options shared by the focus manager methods. */
export interface FocusScopeMoveOptions {
  /** Element from which traversal begins. Defaults to the deep active element. */
  readonly from?: Element | null;

  /**
   * Include focusable elements excluded from sequential Tab order.
   *
   * @default false
   */
  readonly includeProgrammatic?: boolean;

  /**
   * Wrap traversal at the scope boundary.
   *
   * @default false
   */
  readonly wrap?: boolean;

  /**
   * Avoid browser scrolling while focus moves.
   *
   * @default true
   */
  readonly preventScroll?: boolean;

  /** Additional consumer filter applied after native focusability checks. */
  readonly accept?: (element: HTMLElement) => boolean;
}

/** Focus containment, entry, and restoration options for one DOM subtree. */
export interface FocusScopeOptions {
  /** Scope root. It may be `null` during SSR and before mount. */
  readonly root: MaybeRefOrGetter<Element | null | undefined>;

  /**
   * Trap sequential and programmatic focus inside the active, topmost scope.
   *
   * @default false
   */
  readonly contain?: MaybeRefOrGetter<boolean | undefined>;

  /**
   * Focus an initial target when the activated scope receives a mounted root.
   *
   * @default false
   */
  readonly autoFocus?: MaybeRefOrGetter<boolean | undefined>;

  /**
   * Restore focus when the scope deactivates.
   *
   * @default false
   */
  readonly restoreFocus?: MaybeRefOrGetter<boolean | undefined>;

  /** Preferred initial target. It must resolve inside the current scope. */
  readonly initialFocus?: () => HTMLElement | null | undefined;

  /** Explicit restoration target, evaluated at deactivation time. */
  readonly restoreTarget?: () => HTMLElement | null | undefined;

  /** Fallback target when a scope has no tabbable descendants. */
  readonly fallbackFocus?: () => HTMLElement | null | undefined;

  /** Consumer filter applied to automatically discovered focusable descendants. */
  readonly accept?: (element: HTMLElement) => boolean;

  /** Called before automatic entry focus. Call `preventDefault` to retain focus. */
  readonly onMountAutoFocus?: (event: FocusScopeAutoFocusEvent) => void;

  /** Called before automatic restoration. Call `preventDefault` to retain focus. */
  readonly onUnmountAutoFocus?: (event: FocusScopeAutoFocusEvent) => void;
}

/** Stable focus traversal interface for one scope. */
export interface FocusScopeManager {
  readonly focusFirst: (options?: FocusScopeMoveOptions) => HTMLElement | null;
  readonly focusLast: (options?: FocusScopeMoveOptions) => HTMLElement | null;
  readonly focusNext: (options?: FocusScopeMoveOptions) => HTMLElement | null;
  readonly focusPrevious: (options?: FocusScopeMoveOptions) => HTMLElement | null;
}

/** Lifecycle and traversal controller returned by `createFocusScope`. */
export interface FocusScopeController extends FocusScopeManager {
  /** Whether this controller has been activated and not deactivated. */
  readonly isActive: Readonly<ShallowRef<boolean>>;

  /** Activate once, capture restoration context, and attach when a root exists. */
  readonly activate: () => void;

  /** Deactivate once, detach containment, and restore focus when configured. */
  readonly deactivate: () => void;

  /** Re-read the root and recover containment after imperative DOM changes. */
  readonly refresh: () => void;

  /** Permanently release watches and lifecycle ownership. */
  readonly dispose: () => void;
}
