import { computed } from "vue";
import type { ComputedRef, Ref } from "vue";

import { useMediaQuery } from "./media-query.ts";
import type { UseMediaQueryOptions } from "./media-query.ts";

/** Breakpoint width: pixels as a number, or a CSS length in `px`, `rem`, or `em`. */
export type BreakpointValue = number | `${number}px` | `${number}rem` | `${number}em`;

/** Named breakpoint map. */
export type Breakpoints<Name extends string = string> = Readonly<Record<Name, BreakpointValue>>;

/** Tailwind CSS default breakpoints. */
export const breakpointsTailwind = {
  sm: 640,
  md: 768,
  lg: 1024,
  xl: 1280,
  "2xl": 1536,
} as const satisfies Breakpoints;

/** Bootstrap 5 default breakpoints. */
export const breakpointsBootstrapV5 = {
  xs: 0,
  sm: 576,
  md: 768,
  lg: 992,
  xl: 1200,
  xxl: 1400,
} as const satisfies Breakpoints;

/** Names reserved for the helper methods returned by {@link useBreakpoints}. */
export type ReservedBreakpointName =
  | "greaterOrEqual"
  | "greater"
  | "smaller"
  | "smallerOrEqual"
  | "between"
  | "current"
  | "active";

/** Options for {@link useBreakpoints}. */
export interface UseBreakpointsOptions extends Omit<UseMediaQueryOptions, "ssrValue"> {
  /**
   * Viewport width assumed during server rendering (and whenever no media
   * capability exists). Queries are evaluated against it arithmetically, so
   * the server output matches a client of that width.
   *
   * @default undefined (every query is `false` on the server)
   */
  readonly ssrWidth?: number;

  /**
   * Root font size used to convert `rem`/`em` values for `ssrWidth`.
   *
   * @default 16
   */
  readonly rootFontSize?: number;
}

/** Reactive breakpoint helpers returned by {@link useBreakpoints}. */
export type BreakpointsControls<Name extends string> = {
  /** `true` when the viewport is at least this breakpoint (mobile-first). */
  readonly [Key in Name]: Readonly<Ref<boolean>>;
} & {
  /** Viewport width ≥ breakpoint. */
  readonly greaterOrEqual: (name: Name) => Readonly<Ref<boolean>>;
  /** Viewport width > breakpoint. */
  readonly greater: (name: Name) => Readonly<Ref<boolean>>;
  /** Viewport width < breakpoint. */
  readonly smaller: (name: Name) => Readonly<Ref<boolean>>;
  /** Viewport width ≤ breakpoint. */
  readonly smallerOrEqual: (name: Name) => Readonly<Ref<boolean>>;
  /** `lower` ≤ width < `upper`. */
  readonly between: (lower: Name, upper: Name) => ComputedRef<boolean>;
  /** Every breakpoint currently matched (mobile-first), in ascending order. */
  readonly current: ComputedRef<readonly Name[]>;
  /** Largest matched breakpoint, or `null` below the smallest one. */
  readonly active: ComputedRef<Name | null>;
};

/**
 * Reactive, typed breakpoint helpers built on media queries.
 *
 * Breakpoint names are inferred from the map, so typos are compile errors.
 * `greater` and `smaller` use fractional offsets (`width > 767.98px`) so
 * adjacent breakpoints never overlap. With `ssrWidth`, the server evaluates
 * each query arithmetically for deterministic, hydration-stable output.
 * Subscriptions are released with the owning reactive scope.
 *
 * @param breakpoints Named breakpoint map, e.g. {@link breakpointsTailwind}.
 * @param options SSR width, font size, and media capability.
 * @default options {}
 * @returns Per-breakpoint refs plus comparison helpers.
 */
export function useBreakpoints<const Name extends string>(
  breakpoints: Breakpoints<Name> & { readonly [Reserved in ReservedBreakpointName]?: never },
  options: UseBreakpointsOptions = {},
): BreakpointsControls<Name> {
  const rootFontSize = options.rootFontSize ?? 16;
  const isName = (key: string): key is Name => Object.hasOwn(breakpoints, key);
  const names = Object.keys(breakpoints).filter(isName);
  names.sort(
    (left, right) =>
      toPixels(breakpoints[left], rootFontSize) - toPixels(breakpoints[right], rootFontSize),
  );
  const cache = new Map<string, Readonly<Ref<boolean>>>();

  const query = (feature: "min-width" | "max-width", pixels: number): Readonly<Ref<boolean>> => {
    const text = `(${feature}: ${pixels}px)`;
    const cached = cache.get(text);
    if (cached) return cached;
    const ssrWidth = options.ssrWidth;
    const ssrValue =
      ssrWidth === undefined
        ? false
        : feature === "min-width"
          ? ssrWidth >= pixels
          : ssrWidth <= pixels;
    const matches = useMediaQuery(
      text,
      options.host === undefined ? { ssrValue } : { ssrValue, host: options.host },
    );
    cache.set(text, matches);
    return matches;
  };
  const px = (name: Name): number => toPixels(breakpoints[name], rootFontSize);
  const greaterOrEqual = (name: Name): Readonly<Ref<boolean>> => query("min-width", px(name));
  const greater = (name: Name): Readonly<Ref<boolean>> => query("min-width", px(name) + 0.02);
  const smaller = (name: Name): Readonly<Ref<boolean>> => query("max-width", px(name) - 0.02);
  const smallerOrEqual = (name: Name): Readonly<Ref<boolean>> => query("max-width", px(name));

  // Every key is assigned by the loop below before the record is read.
  const perName = {} as Record<Name, Readonly<Ref<boolean>>>;
  for (const name of names) perName[name] = greaterOrEqual(name);
  const current = computed<readonly Name[]>(() => names.filter((name) => perName[name].value));

  return {
    ...perName,
    greaterOrEqual,
    greater,
    smaller,
    smallerOrEqual,
    between: (lower, upper) => {
      const above = greaterOrEqual(lower);
      const below = smaller(upper);
      return computed(() => above.value && below.value);
    },
    current,
    active: computed(() => current.value.at(-1) ?? null),
  };
}

/**
 * Convert a {@link BreakpointValue} to CSS pixels.
 *
 * @param value Number of pixels or a `px`/`rem`/`em` length.
 * @param rootFontSize Pixels per `rem`/`em`.
 * @default rootFontSize 16
 * @returns Width in CSS pixels.
 */
export function toPixels(value: BreakpointValue, rootFontSize = 16): number {
  if (typeof value === "number") return value;
  const amount = Number.parseFloat(value);
  return value.endsWith("px") ? amount : amount * rootFontSize;
}
