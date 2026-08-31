import type { ShallowRef } from "vue";

/** One measured border-box change reported by a size observer. */
export interface SizeObserverEntry {
  /** Observed element that changed size. */
  readonly target: Element;

  /** Inline-axis extent in CSS pixels for the configured box. */
  readonly width: number;

  /** Block-axis extent in CSS pixels for the configured box. */
  readonly height: number;
}

/** Options shared by {@link createSizeObserver} and {@link useSizeObserver}. */
export interface SizeObserverOptions {
  /**
   * Box model reported for every observed target.
   *
   * @default "border-box"
   */
  readonly box?: "border-box" | "content-box";

  /** Called with each batch of size changes, in observation order. */
  readonly onResize: (entries: readonly SizeObserverEntry[]) => void;
}

/** SSR-safe `ResizeObserver` wrapper for a stable set of targets. */
export interface SizeObserverController {
  /** Whether the platform provides `ResizeObserver`. Always `false` during SSR. */
  readonly isSupported: boolean;

  /** Number of currently observed targets. */
  readonly observedCount: Readonly<ShallowRef<number>>;

  /** Start observing one element. Repeated calls for one element are idempotent. */
  readonly observe: (target: Element) => void;

  /** Stop observing one element. Unknown elements are ignored. */
  readonly unobserve: (target: Element) => void;

  /** Stop observing every element while keeping the controller usable. */
  readonly disconnect: () => void;

  /** Disconnect and freeze the controller. Further observation throws. */
  readonly dispose: () => void;
}

/** One visibility change reported by a visibility observer. */
export interface VisibilityObserverEntry {
  /** Observed element whose intersection changed. */
  readonly target: Element;

  /** Whether the target intersects the observation root. */
  readonly isIntersecting: boolean;

  /** Fraction of the target inside the root, from `0` to `1`. */
  readonly intersectionRatio: number;
}

/** Options shared by {@link createVisibilityObserver} and {@link useVisibilityObserver}. */
export interface VisibilityObserverOptions {
  /**
   * Intersection root. `null` observes against the viewport.
   *
   * @default null
   */
  readonly root?: Element | Document | null;

  /**
   * Margin applied to the root bounding box before intersection.
   *
   * @default "0px"
   */
  readonly rootMargin?: string;

  /**
   * Intersection ratios at which changes are reported.
   *
   * @default 0
   */
  readonly threshold?: number | readonly number[];

  /** Called with each batch of visibility changes, in observation order. */
  readonly onVisibilityChange: (entries: readonly VisibilityObserverEntry[]) => void;
}

/** SSR-safe `IntersectionObserver` wrapper for a stable set of targets. */
export interface VisibilityObserverController {
  /** Whether the platform provides `IntersectionObserver`. Always `false` during SSR. */
  readonly isSupported: boolean;

  /** Number of currently observed targets. */
  readonly observedCount: Readonly<ShallowRef<number>>;

  /** Start observing one element. Repeated calls for one element are idempotent. */
  readonly observe: (target: Element) => void;

  /** Stop observing one element. Unknown elements are ignored. */
  readonly unobserve: (target: Element) => void;

  /** Stop observing every element while keeping the controller usable. */
  readonly disconnect: () => void;

  /** Disconnect and freeze the controller. Further observation throws. */
  readonly dispose: () => void;
}
