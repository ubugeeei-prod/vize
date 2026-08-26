import type { MaybeRefOrGetter, ShallowRef } from "vue";

/** Lifecycle phase published by a presence controller. */
export type PresenceStatus = "entering" | "exiting" | "present" | "unmounted";

/** Options shared by {@link createPresence} and {@link usePresence}. */
export interface PresenceOptions {
  /**
   * Whether the content should occupy the tree.
   * Reactive values are resolved for every presence transition.
   *
   * @default false
   */
  readonly present?: MaybeRefOrGetter<boolean | undefined>;

  /**
   * Skip enter and exit animation when `prefers-reduced-motion: reduce` matches.
   *
   * @default true
   */
  readonly respectReducedMotion?: MaybeRefOrGetter<boolean | undefined>;

  /** Called after enter animation completes or is skipped. */
  readonly onEnterComplete?: () => void;

  /** Called after exit animation completes or is skipped. */
  readonly onExitComplete?: () => void;
}

/** Native handlers to spread onto the present host. */
export interface PresenceProps {
  readonly onAnimationend: (event: AnimationEvent) => void;
  readonly onTransitionend: (event: TransitionEvent) => void;
}

/** Stateful presence normalizer with explicit animation completion. */
export interface PresenceController {
  /** Whether the slot should remain mounted, including while exiting. */
  readonly isPresent: Readonly<ShallowRef<boolean>>;

  /** Current enter/exit phase. */
  readonly status: Readonly<ShallowRef<PresenceStatus>>;

  /** Stable animation completion handlers to merge onto one host. */
  readonly presenceProps: Readonly<PresenceProps>;

  /** Advance from entering to present, or from exiting to unmounted. */
  readonly completeAnimation: () => void;

  /** Release watchers and freeze the controller. */
  readonly dispose: () => void;
}
