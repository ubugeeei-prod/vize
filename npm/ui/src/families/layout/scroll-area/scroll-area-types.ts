import type { PrimitiveAs, PrimitiveElement } from "../../foundations/primitive/primitive.ts";

/** Native element, custom element, or component accepted by the ScrollArea root. */
export type ScrollAreaAs = PrimitiveAs;

/** Logical scroll axis exposed by {@link ScrollArea}. */
export type ScrollAreaOrientation = "vertical" | "horizontal" | "both";

/** Reading direction reflected on the scroll viewport. */
export type ScrollAreaDirection = "ltr" | "rtl";

/** Native CSS scroll behavior applied to programmatic scrolling. */
export type ScrollAreaScrollBehavior = "auto" | "smooth";

/** Native CSS overscroll policy applied to the viewport. */
export type ScrollAreaOverscrollBehavior = "auto" | "contain" | "none";

/** Native CSS scrollbar gutter policy. */
export type ScrollAreaScrollbarGutter = "auto" | "stable" | "stable both-edges";

/** Native CSS scrollbar width hook. */
export type ScrollAreaScrollbarWidth = "auto" | "thin" | "none";

/** Native CSS length, percentage, keyword, or custom property accepted by ScrollArea. */
export type ScrollAreaLength = string | number;

/** CSS-ready value published after numeric lengths are normalized. */
export type ScrollAreaResolvedLength = string;

/** Native CSS overflow value used by the viewport. */
export type ScrollAreaOverflow = "auto" | "hidden";

/** Stable state token for ScrollArea styling and tests. */
export type ScrollAreaState = "scrollable";

/** Rendered root value exposed by {@link ScrollArea}. */
export type ScrollAreaRootElement = PrimitiveElement;

/** Public props accepted by the ScrollArea primitive. */
export interface ScrollAreaProps {
  /**
   * Native element, custom element, or component to render as the root.
   *
   * @default "div"
   */
  readonly as?: ScrollAreaAs;

  /**
   * Logical scroll axis controlled by the native viewport overflow.
   *
   * @default "vertical"
   */
  readonly orientation?: ScrollAreaOrientation;

  /**
   * Reading direction reflected with `dir` and `data-dir`.
   *
   * @default "ltr"
   */
  readonly dir?: ScrollAreaDirection;

  /**
   * Make the viewport keyboard-focusable for standalone scrollable regions.
   *
   * @default false
   */
  readonly focusable?: boolean;

  /**
   * Root block size. Numbers resolve to px lengths.
   *
   * @default "auto"
   */
  readonly blockSize?: ScrollAreaLength;

  /**
   * Root inline size. Numbers resolve to px lengths.
   *
   * @default "auto"
   */
  readonly inlineSize?: ScrollAreaLength;

  /**
   * Root max block size. Numbers resolve to px lengths.
   *
   * @default "none"
   */
  readonly maxBlockSize?: ScrollAreaLength;

  /**
   * Root max inline size. Numbers resolve to px lengths.
   *
   * @default "none"
   */
  readonly maxInlineSize?: ScrollAreaLength;

  /**
   * Native overscroll policy for the viewport.
   *
   * @default "auto"
   */
  readonly overscrollBehavior?: ScrollAreaOverscrollBehavior;

  /**
   * Native scroll behavior for programmatic scrolling.
   *
   * @default "auto"
   */
  readonly scrollBehavior?: ScrollAreaScrollBehavior;

  /**
   * Native scrollbar gutter policy.
   *
   * @default "auto"
   */
  readonly scrollbarGutter?: ScrollAreaScrollbarGutter;

  /**
   * Native scrollbar width hook.
   *
   * @default "auto"
   */
  readonly scrollbarWidth?: ScrollAreaScrollbarWidth;

  /**
   * Accessible name for the scroll viewport; also promotes it to `role="region"`.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids that label the scroll viewport.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;

  /**
   * Space-separated ids that describe the scroll viewport.
   *
   * @default undefined
   */
  readonly ariaDescribedby?: string;
}

/** Emits published by the ScrollArea component. */
export interface ScrollAreaEmits {
  /** Fired when the native viewport dispatches a scroll event. */
  scroll: [nativeEvent: Event];
}

/** Inline style hooks applied to the rendered ScrollArea root. */
export interface ScrollAreaStyle {
  /** Consumer-overridable root block size hook. */
  readonly "--vize-ui-scroll-area-block-size": ScrollAreaResolvedLength;

  /** Consumer-overridable root inline size hook. */
  readonly "--vize-ui-scroll-area-inline-size": ScrollAreaResolvedLength;

  /** Consumer-overridable root max block size hook. */
  readonly "--vize-ui-scroll-area-max-block-size": ScrollAreaResolvedLength;

  /** Consumer-overridable root max inline size hook. */
  readonly "--vize-ui-scroll-area-max-inline-size": ScrollAreaResolvedLength;

  /** Consumer-overridable viewport horizontal overflow hook. */
  readonly "--vize-ui-scroll-area-overflow-x": ScrollAreaOverflow;

  /** Consumer-overridable viewport vertical overflow hook. */
  readonly "--vize-ui-scroll-area-overflow-y": ScrollAreaOverflow;

  /** Consumer-overridable viewport overscroll hook. */
  readonly "--vize-ui-scroll-area-overscroll-behavior": ScrollAreaOverscrollBehavior;

  /** Consumer-overridable viewport scroll behavior hook. */
  readonly "--vize-ui-scroll-area-scroll-behavior": ScrollAreaScrollBehavior;

  /** Consumer-overridable native scrollbar gutter hook. */
  readonly "--vize-ui-scroll-area-scrollbar-gutter": ScrollAreaScrollbarGutter;

  /** Consumer-overridable native scrollbar width hook. */
  readonly "--vize-ui-scroll-area-scrollbar-width": ScrollAreaScrollbarWidth;
}

/** Normalized ARIA IDREF state rendered by ScrollArea. */
export interface ScrollAreaAriaState {
  /** Normalized `aria-label` value, or `undefined` when absent. */
  readonly ariaLabel: string | undefined;

  /** Normalized `aria-labelledby` value, or `undefined` when absent. */
  readonly ariaLabelledby: string | undefined;

  /** Normalized `aria-describedby` value, or `undefined` when absent. */
  readonly ariaDescribedby: string | undefined;
}

/** State exposed to the default ScrollArea slot. */
export interface ScrollAreaSlotState extends ScrollAreaAriaState {
  /** Rendered root host. */
  readonly as: ScrollAreaAs;

  /** Logical scroll axis. */
  readonly orientation: ScrollAreaOrientation;

  /** Reading direction reflected on the viewport. */
  readonly dir: ScrollAreaDirection;

  /** Whether the viewport is keyboard-focusable. */
  readonly focusable: boolean;

  /** Resolved root block size. */
  readonly blockSize: ScrollAreaResolvedLength;

  /** Resolved root inline size. */
  readonly inlineSize: ScrollAreaResolvedLength;

  /** Resolved root max block size. */
  readonly maxBlockSize: ScrollAreaResolvedLength;

  /** Resolved root max inline size. */
  readonly maxInlineSize: ScrollAreaResolvedLength;

  /** Native viewport horizontal overflow value. */
  readonly overflowX: ScrollAreaOverflow;

  /** Native viewport vertical overflow value. */
  readonly overflowY: ScrollAreaOverflow;

  /** Native viewport overscroll policy. */
  readonly overscrollBehavior: ScrollAreaOverscrollBehavior;

  /** Native viewport scroll behavior. */
  readonly scrollBehavior: ScrollAreaScrollBehavior;

  /** Native viewport scrollbar gutter policy. */
  readonly scrollbarGutter: ScrollAreaScrollbarGutter;

  /** Native viewport scrollbar width hook. */
  readonly scrollbarWidth: ScrollAreaScrollbarWidth;

  /** Whether the viewport has an accessible name. */
  readonly labelled: boolean;

  /** Whether the viewport has an accessible description reference. */
  readonly described: boolean;

  /** Stable state token for styling and tests. */
  readonly state: ScrollAreaState;

  /** Native CSS custom property hooks applied to the root. */
  readonly style: ScrollAreaStyle;
}

/** Resolved layout state published by {@link ScrollArea}. */
export type ScrollAreaResolvedLayout = Omit<
  ScrollAreaSlotState,
  "ariaDescribedby" | "ariaLabel" | "ariaLabelledby" | "as" | "described" | "labelled"
>;

/** Public instance state exposed by the ScrollArea primitive. */
export interface ScrollAreaExpose extends ScrollAreaSlotState {
  /** Rendered root element or component instance. */
  readonly root: ScrollAreaRootElement | null;

  /** Rendered native scroll viewport. */
  readonly viewport: HTMLDivElement | null;

  /** Move DOM focus to the native viewport. */
  readonly focus: (options?: FocusOptions) => void;

  /** Scroll the native viewport using the platform `scrollTo` API. */
  readonly scrollTo: (options?: ScrollToOptions) => void;

  /** Scroll the native viewport using the platform `scrollBy` API. */
  readonly scrollBy: (options?: ScrollToOptions) => void;
}
