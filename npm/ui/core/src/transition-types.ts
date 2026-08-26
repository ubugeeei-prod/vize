import type { MaybeRefOrGetter } from "vue";

import type { PresenceController, PresenceOptions } from "./presence-types.ts";

/** Options shared by {@link createTransition} and {@link useTransition}. */
export interface TransitionOptions extends PresenceOptions {
  /**
   * Extra milliseconds added to the computed motion duration before auto-complete.
   *
   * @default 0
   */
  readonly timeoutPadding?: MaybeRefOrGetter<number | undefined>;
}

/** Presence controller that completes enter/exit from computed CSS motion. */
export interface TransitionController extends PresenceController {
  /** Bind the host whose computed animation and transition durations are read. */
  readonly setElement: (element: Element | null) => void;
}
