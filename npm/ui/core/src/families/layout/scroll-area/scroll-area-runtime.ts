import type {
  ScrollAreaAriaState,
  ScrollAreaLength,
  ScrollAreaOrientation,
  ScrollAreaOverflow,
  ScrollAreaResolvedLayout,
  ScrollAreaResolvedLength,
  ScrollAreaStyle,
} from "./scroll-area-types.ts";

const defaultSize = "auto";
const defaultMaxSize = "none";

interface ScrollAreaAriaOptions {
  readonly ariaDescribedby?: string | undefined;
  readonly ariaLabel?: string | undefined;
  readonly ariaLabelledby?: string | undefined;
}

interface ScrollAreaLayoutOptions {
  readonly blockSize?: ScrollAreaLength | undefined;
  readonly dir?: ScrollAreaResolvedLayout["dir"] | undefined;
  readonly focusable?: boolean | undefined;
  readonly inlineSize?: ScrollAreaLength | undefined;
  readonly maxBlockSize?: ScrollAreaLength | undefined;
  readonly maxInlineSize?: ScrollAreaLength | undefined;
  readonly orientation?: ScrollAreaOrientation | undefined;
  readonly overscrollBehavior?: ScrollAreaResolvedLayout["overscrollBehavior"] | undefined;
  readonly scrollBehavior?: ScrollAreaResolvedLayout["scrollBehavior"] | undefined;
  readonly scrollbarGutter?: ScrollAreaResolvedLayout["scrollbarGutter"] | undefined;
  readonly scrollbarWidth?: ScrollAreaResolvedLayout["scrollbarWidth"] | undefined;
}

/** Normalize one optional ARIA token or IDREF list without inventing ids. */
export function normalizeScrollAreaAriaToken(value: string | undefined): string | undefined {
  const normalized = value?.trim().replaceAll(/\s+/g, " ");
  return normalized === "" ? undefined : normalized;
}

/** Resolve ScrollArea accessibility props into native viewport ARIA attributes. */
export function resolveScrollAreaAria(options: ScrollAreaAriaOptions): ScrollAreaAriaState {
  return {
    ariaDescribedby: normalizeScrollAreaAriaToken(options.ariaDescribedby),
    ariaLabel: normalizeScrollAreaAriaToken(options.ariaLabel),
    ariaLabelledby: normalizeScrollAreaAriaToken(options.ariaLabelledby),
  };
}

/** Convert numeric CSS lengths to px while keeping authored strings intact. */
export function normalizeScrollAreaLength(
  value: ScrollAreaLength | undefined,
  fallback: ScrollAreaResolvedLength,
): ScrollAreaResolvedLength {
  if (typeof value === "number") return Number.isFinite(value) ? `${value}px` : fallback;
  return value === undefined ? fallback : value;
}

/** Resolve the native overflow pair for one logical scroll orientation. */
export function resolveScrollAreaOverflow(orientation: ScrollAreaOrientation): {
  readonly overflowX: ScrollAreaOverflow;
  readonly overflowY: ScrollAreaOverflow;
} {
  if (orientation === "horizontal") return { overflowX: "auto", overflowY: "hidden" };
  if (orientation === "both") return { overflowX: "auto", overflowY: "auto" };
  return { overflowX: "hidden", overflowY: "auto" };
}

/** Resolve ScrollArea props into CSS custom properties and slot/expose state. */
export function resolveScrollAreaLayout(
  options: ScrollAreaLayoutOptions,
): ScrollAreaResolvedLayout {
  const orientation = options.orientation ?? "vertical";
  const { overflowX, overflowY } = resolveScrollAreaOverflow(orientation);
  const blockSize = normalizeScrollAreaLength(options.blockSize, defaultSize);
  const inlineSize = normalizeScrollAreaLength(options.inlineSize, defaultSize);
  const maxBlockSize = normalizeScrollAreaLength(options.maxBlockSize, defaultMaxSize);
  const maxInlineSize = normalizeScrollAreaLength(options.maxInlineSize, defaultMaxSize);
  const overscrollBehavior = options.overscrollBehavior ?? "auto";
  const scrollBehavior = options.scrollBehavior ?? "auto";
  const scrollbarGutter = options.scrollbarGutter ?? "auto";
  const scrollbarWidth = options.scrollbarWidth ?? "auto";
  const style: ScrollAreaStyle = {
    "--vize-ui-scroll-area-block-size": blockSize,
    "--vize-ui-scroll-area-inline-size": inlineSize,
    "--vize-ui-scroll-area-max-block-size": maxBlockSize,
    "--vize-ui-scroll-area-max-inline-size": maxInlineSize,
    "--vize-ui-scroll-area-overscroll-behavior": overscrollBehavior,
    "--vize-ui-scroll-area-overflow-x": overflowX,
    "--vize-ui-scroll-area-overflow-y": overflowY,
    "--vize-ui-scroll-area-scroll-behavior": scrollBehavior,
    "--vize-ui-scroll-area-scrollbar-gutter": scrollbarGutter,
    "--vize-ui-scroll-area-scrollbar-width": scrollbarWidth,
  };

  return {
    blockSize,
    dir: options.dir ?? "ltr",
    focusable: options.focusable === true,
    inlineSize,
    maxBlockSize,
    maxInlineSize,
    orientation,
    overflowX,
    overflowY,
    overscrollBehavior,
    scrollBehavior,
    scrollbarGutter,
    scrollbarWidth,
    state: "scrollable",
    style,
  };
}
