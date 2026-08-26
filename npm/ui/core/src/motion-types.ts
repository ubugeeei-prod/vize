/** Named steps on the motion duration scale. */
export type MotionDurationToken = "instant" | "fast" | "base" | "slow" | "deliberate";

/** Named steps on the motion delay scale. */
export type MotionDelayToken = "none" | "hint" | "stagger";

/** Named easing curves. */
export type MotionEasingToken = "standard" | "decelerate" | "accelerate" | "emphasized" | "linear";

/** Enter/exit recipes understood by the presence and transition primitives. */
export type MotionEnterExitRecipe = "fade" | "scale" | "slide";

/** Every recipe name accepted in the space-separated `data-vize-motion` attribute. */
export type MotionRecipe = MotionEnterExitRecipe | "enter" | "move" | "pulse" | "shake";

/** Recipe hook properties that retune one motion phase without forking components. */
export type MotionRecipeHook =
  | "enter-duration"
  | "enter-easing"
  | "exit-duration"
  | "exit-easing"
  | "move-duration"
  | "move-easing"
  | "emphasis-duration"
  | "emphasis-easing"
  | "slide-distance"
  | "scale-from";

/** Suffix of one `--vize-ui-motion-*` custom property. */
export type MotionTokenName =
  | `duration-${MotionDurationToken}`
  | `delay-${MotionDelayToken}`
  | `ease-${MotionEasingToken}`
  | MotionRecipeHook;

/** Custom-property overrides applied by {@link setMotionTokens}. */
export type MotionTokenOverrides = {
  readonly [Name in MotionTokenName]?: string;
};

/** Options accepted by {@link startViewTransition}. */
export interface StartViewTransitionOptions {
  /**
   * Skip the native crossfade and apply the update directly when
   * `prefers-reduced-motion: reduce` matches.
   *
   * @default true
   */
  readonly respectReducedMotion?: boolean;
}

/** Normalized handle returned by {@link startViewTransition} on every platform. */
export interface ViewTransitionHandle {
  /** Whether a native document view transition is driving the update. */
  readonly native: boolean;

  /** Resolves when the transition and its animations have finished. */
  readonly finished: Promise<void>;

  /**
   * Resolves when transition animations are about to start. Rejects when the
   * browser skips the transition; the rejection is pre-handled so ignoring
   * `ready` never surfaces an unhandled rejection.
   */
  readonly ready: Promise<void>;

  /** Resolves when the DOM update callback has run to completion. */
  readonly updateCallbackDone: Promise<void>;

  /** Skip remaining animations while still applying the DOM update. */
  readonly skipTransition: () => void;
}
