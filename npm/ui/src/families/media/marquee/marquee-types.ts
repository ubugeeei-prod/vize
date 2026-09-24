/** Scroll direction of the marquee track. */
export type MarqueeDirection = "down" | "left" | "right" | "up";

/** Axis derived from {@link MarqueeDirection}. */
export type MarqueeOrientation = "horizontal" | "vertical";

/** Animation state mirrored through `data-state`. */
export type MarqueeState = "paused" | "running";

/**
 * Why a marquee is paused.
 *
 * - `user`: the play intent is off (pause button, `v-model:playing`, or `pause()`).
 * - `hover`: a mouse or pen pointer rests on the marquee.
 * - `focus`: keyboard focus is inside the marquee.
 * - `reduced-motion`: `prefers-reduced-motion: reduce` matches and the user has not opted in.
 */
export type MarqueePauseReason = "focus" | "hover" | "reduced-motion" | "user";

/** Localizable strings used by the marquee parts. */
export interface MarqueeMessages {
  /** Accessible name of the pause button while the marquee moves. */
  readonly pause: string;

  /** Accessible name of the pause button while the marquee is stopped. */
  readonly play: string;
}

/** Partial overrides accepted by the `messages` prop. */
export type MarqueeMessageOverrides = Partial<MarqueeMessages>;

/** Default English messages. */
export const defaultMarqueeMessages: MarqueeMessages = Object.freeze({
  pause: "Pause scrolling content",
  play: "Play scrolling content",
});

/** Measured geometry along the scroll axis, in CSS pixels. */
export interface MarqueeMeasurement {
  /** Distance between the starts of two consecutive copies (content size plus gap). */
  readonly distance: number;

  /** Visible length of the marquee root. */
  readonly viewport: number;
}

/** State exposed to every marquee slot. */
export interface MarqueeSlotState {
  /** Animation state. */
  readonly state: MarqueeState;

  /** Highest-priority pause reason, or `null` while running. */
  readonly pauseReason: MarqueePauseReason | null;

  /** Whether the play intent is on (`v-model:playing`). */
  readonly playing: boolean;

  /** Scroll direction. */
  readonly direction: MarqueeDirection;

  /** Number of rendered copies (the first is the accessible one). */
  readonly copies: number;

  /** Seconds for one full loop, or `null` until measured. */
  readonly duration: number | null;
}

/** Public instance exposed by MarqueeRoot. */
export interface MarqueeRootExpose extends MarqueeSlotState {
  /** Rendered root element. */
  readonly element: HTMLDivElement | null;

  /** Turn the play intent on and opt in to motion under reduced-motion preferences. */
  readonly play: () => boolean;

  /** Turn the play intent off. */
  readonly pause: () => boolean;

  /** Toggle the play intent. */
  readonly toggle: () => boolean;
}

/** Public instance exposed by MarqueeContent. */
export interface MarqueeContentExpose {
  /** Rendered track element that the consumer animates. */
  readonly element: HTMLDivElement | null;

  /** Re-measure geometry, e.g. after fonts load. */
  readonly measure: () => void;
}

/** Public instance exposed by MarqueePauseButton. */
export interface MarqueePauseButtonExpose {
  /** Rendered native button. */
  readonly element: HTMLButtonElement | null;
}

/** Merge partial message overrides over the English defaults. */
export function resolveMarqueeMessages(
  overrides: MarqueeMessageOverrides | undefined,
): MarqueeMessages {
  return { ...defaultMarqueeMessages, ...overrides };
}
