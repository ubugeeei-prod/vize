/** Values accepted by the native `aria-current` attribute. */
export type LinkAriaCurrent = boolean | "date" | "location" | "page" | "step" | "time";

/** Values accepted by the native anchor `download` attribute. */
export type LinkDownload = boolean | string;

/** Public props accepted by the Link primitive. */
export interface LinkProps {
  /**
   * Consumer-owned anchor id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Native navigation target for the anchor.
   *
   * @default undefined
   */
  readonly href?: string;

  /**
   * Native browsing context target.
   *
   * @default undefined
   */
  readonly target?: string;

  /**
   * Native relationship tokens such as `noopener` or `external`.
   *
   * @default undefined
   */
  readonly rel?: string;

  /**
   * Native download hint. `true` renders the boolean attribute.
   *
   * @default undefined
   */
  readonly download?: LinkDownload;

  /**
   * Remove link activation and sequential keyboard focus.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Apply native inertness while also suppressing anchor activation in older runtimes.
   *
   * @default false
   */
  readonly inert?: boolean;

  /**
   * Mark the link as the current item in a set.
   *
   * @default undefined
   */
  readonly ariaCurrent?: LinkAriaCurrent;
}

/** State exposed to the default slot. */
export interface LinkSlotState {
  /** Whether the link is explicitly disabled. */
  readonly disabled: boolean;

  /** Whether native inertness is requested. */
  readonly inert: boolean;

  /** Whether the link cannot be activated by the user. */
  readonly unavailable: boolean;
}

/** Methods exposed by the link component instance. */
export interface LinkExpose {
  /** Move focus to the native anchor. */
  readonly focus: (options?: FocusOptions) => void;
}
