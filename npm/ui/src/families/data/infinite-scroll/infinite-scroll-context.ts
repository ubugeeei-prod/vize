import type { ComputedRef, Ref } from "vue";

import { createContext } from "../../foundations/context/context.ts";
import type {
  InfiniteScrollObserverRoot,
  InfiniteScrollSlotState,
  InfiniteScrollState,
  InfiniteScrollTrigger,
} from "./infinite-scroll-types.ts";

/** Shared state and actions for the InfiniteScroll compound parts. */
export interface InfiniteScrollContextValue {
  /** Deterministic root id used by `aria-controls`. */
  readonly id: ComputedRef<string>;

  /** Current loading state. */
  readonly state: ComputedRef<InfiniteScrollState>;

  /** Slot state shared by every part. */
  readonly slotState: ComputedRef<InfiniteScrollSlotState>;

  /** Whether the root renders the APG feed pattern. */
  readonly feed: ComputedRef<boolean>;

  /** Announced set size for feed articles. */
  readonly setSize: ComputedRef<number>;

  /** Intersection root selection. */
  readonly scrollRoot: ComputedRef<InfiniteScrollObserverRoot>;

  /** Intersection margin. */
  readonly rootMargin: ComputedRef<string>;

  /** Rendered root element, used when `scrollRoot` is `self`. */
  readonly element: Readonly<Ref<HTMLDivElement | null>>;

  /** Increments whenever the sentinel should re-check its visibility. */
  readonly refreshToken: Readonly<Ref<number>>;

  /** Request the next page. Reports whether a request started. */
  readonly request: (trigger: InfiniteScrollTrigger) => boolean;
}

export const infiniteScrollContext = createContext<InfiniteScrollContextValue>("InfiniteScroll");
