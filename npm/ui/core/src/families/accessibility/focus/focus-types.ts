import type { MaybeRefOrGetter, ShallowRef } from "vue";

/** Scope used to determine whether a host owns the focused element. */
export type FocusMode = "target" | "within";

/** Why a focus observer entered or left its active state. */
export type FocusChangeReason = "disabled" | "focus" | "manual" | "refresh";

/** Immutable, renderer-independent focus lifecycle snapshot. */
export interface FocusEvent {
  /** Lifecycle phase represented by this snapshot. */
  readonly type: "blur" | "focus";

  /** Host element that owns this observer. */
  readonly target: Element;

  /** Focused host or descendant when available. */
  readonly focusedTarget: Element | null;

  /** Destination supplied by a native blur event when available. */
  readonly relatedTarget: Element | null;

  /** Native event responsible for this phase, or `null` for synthetic settlement. */
  readonly originalEvent: globalThis.FocusEvent | null;

  /** Whether focus is currently expected to receive a visible indicator. */
  readonly isFocusVisible: boolean;

  /** Ownership mode used by this observer. */
  readonly mode: FocusMode;

  /** Native or lifecycle operation responsible for this snapshot. */
  readonly reason: FocusChangeReason;
}

/** Native handlers to merge onto one focus host. */
export interface FocusProps {
  /** Direct-target focus handler; present in `target` mode. */
  readonly onFocus?: (event: globalThis.FocusEvent) => void;

  /** Direct-target blur handler; present in `target` mode. */
  readonly onBlur?: (event: globalThis.FocusEvent) => void;

  /** Bubbling focus-entry handler; present in `within` mode. */
  readonly onFocusin?: (event: globalThis.FocusEvent) => void;

  /** Bubbling focus-exit handler; present in `within` mode. */
  readonly onFocusout?: (event: globalThis.FocusEvent) => void;
}

/** Options for {@link createFocus}. */
export interface FocusOptions {
  /**
   * Observe only the host or the host and its composed descendants.
   *
   * @default "target"
   */
  readonly mode?: FocusMode;

  /**
   * Suppress focus ownership and settle an active observer when this becomes true.
   *
   * @default false
   */
  readonly isDisabled?: MaybeRefOrGetter<boolean | undefined>;

  /**
   * Treat owned focus as visibly focused regardless of the current modality.
   * This is intended for elements focused programmatically during mount.
   *
   * @default false
   */
  readonly autoFocus?: boolean;

  /** Called after focus ownership is acquired. */
  readonly onFocus?: (event: FocusEvent) => void;

  /** Called after focus ownership is released. */
  readonly onBlur?: (event: FocusEvent) => void;

  /** Called after each distinct focus-state transition. */
  readonly onFocusChange?: (isFocused: boolean, event: FocusEvent) => void;
}

/** Options for {@link createFocusWithin}. */
export type FocusWithinOptions = Omit<FocusOptions, "mode">;

/** Options for a direct-target focus ring. */
export type FocusRingOptions = Omit<FocusOptions, "mode">;

/** Reactive focus observer with explicit DOM ownership. */
export interface FocusController {
  /** Whether the host currently owns focus according to {@link FocusOptions.mode}. */
  readonly isFocused: Readonly<ShallowRef<boolean>>;

  /** Whether the current focus should expose a visible focus indicator. */
  readonly isFocusVisible: Readonly<ShallowRef<boolean>>;

  /** Stable native handlers to merge onto the host. */
  readonly focusProps: Readonly<FocusProps>;

  /**
   * Reconcile state against the host's current composed-tree active element.
   *
   * @returns `true` when state changed.
   * @throws An error carrying `VIZE_UI_FOCUS_DISPOSED` after disposal.
   */
  readonly refresh: (target: Element) => boolean;

  /**
   * Settle current ownership without moving browser focus.
   *
   * @returns `true` when a focused state was settled.
   * @throws An error carrying `VIZE_UI_FOCUS_DISPOSED` after disposal.
   */
  readonly cancel: () => boolean;

  /** Release modality and document listeners. Safe to call repeatedly. */
  readonly dispose: () => void;
}

/** Direct-target controller returned by {@link createFocusRing}. */
export type FocusRingController = FocusController;
