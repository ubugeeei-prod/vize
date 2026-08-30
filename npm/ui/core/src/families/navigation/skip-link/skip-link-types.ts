/** Hash-fragment destination accepted by the SkipLink primitive. */
export type SkipLinkHref = `#${string}`;

/** Mounted availability and focus state for a skip link. */
export type SkipLinkState = "focused" | "idle" | "invalid";

/** Public props accepted by the SkipLink primitive. */
export interface SkipLinkProps {
  /**
   * Consumer-owned anchor id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Same-document fragment destination. Invalid runtime values remove native navigation.
   *
   * @default "#main"
   */
  readonly href?: SkipLinkHref;

  /**
   * Move DOM focus to the fragment target after activation.
   *
   * @default true
   */
  readonly focusTarget?: boolean;
}

/** Result of an imperative or activation-driven target focus attempt. */
export interface SkipLinkFocusResult {
  /** Same-document target element resolved from the current fragment. */
  readonly target: HTMLElement | null;

  /** Whether focus moved to the target element. */
  readonly focused: boolean;
}

/** Activation payload emitted after a valid skip link click. */
export interface SkipLinkActivation extends SkipLinkFocusResult {
  /** Validated hash-fragment href used for native navigation. */
  readonly href: SkipLinkHref;

  /** Target id resolved from {@link SkipLinkActivation.href}. */
  readonly targetId: string;
}

/** State exposed to the default slot. */
export interface SkipLinkSlotState {
  /** Whether the link itself currently owns focus. */
  readonly focused: boolean;

  /** Validated href rendered on the native anchor. */
  readonly href: SkipLinkHref | undefined;

  /** Mounted availability and focus state. */
  readonly state: SkipLinkState;

  /** Target id resolved from the validated hash-fragment href. */
  readonly targetId: string | undefined;

  /** Whether runtime validation removed native navigation. */
  readonly unavailable: boolean;
}

/** Methods and state exposed by the skip link component instance. */
export interface SkipLinkExpose extends SkipLinkSlotState {
  /** Native anchor element rendered by the primitive. */
  readonly element: HTMLAnchorElement | null;

  /** Move focus to the native anchor. */
  readonly focus: (options?: FocusOptions) => void;

  /** Resolve the current same-document target, or `null` when absent. */
  readonly getTarget: () => HTMLElement | null;

  /** Move focus to the current same-document target. */
  readonly focusTarget: (options?: FocusOptions) => SkipLinkFocusResult;
}
