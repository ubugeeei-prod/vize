import {
  computed,
  getCurrentScope,
  isRef,
  onMounted,
  onScopeDispose,
  shallowRef,
  toValue,
} from "vue";
import type { MaybeRefOrGetter } from "vue";

import type {
  BreakpointHost,
  BreakpointMap,
  BreakpointState,
  UseBreakpointOptions,
} from "./responsive-types.ts";

const setupDiagnostic = "VIZE_UI_RESPONSIVE_SETUP";

/** Tailwind CSS default breakpoints, used when no map is given. */
export const defaultBreakpoints = {
  sm: 640,
  md: 768,
  lg: 1024,
  xl: 1280,
  "2xl": 1536,
} as const satisfies BreakpointMap;

/**
 * Resolve the largest breakpoint reached by `width`.
 *
 * Pure and deterministic: used for SSR (with `ssrWidth`) and after mount.
 *
 * @param breakpoints Named minimum widths.
 * @param width Viewport width, or `null` when unknown.
 * @returns The active breakpoint name, or `null`.
 */
export function resolveActiveBreakpoint<Name extends string>(
  breakpoints: BreakpointMap<Name>,
  width: number | null,
): Name | null {
  if (width === null) return null;
  let active: Name | null = null;
  let best = Number.NEGATIVE_INFINITY;
  for (const [name, minimum] of Object.entries<number>(breakpoints)) {
    if (width >= minimum && minimum > best && isName(breakpoints, name)) {
      active = name;
      best = minimum;
    }
  }
  return active;
}

/**
 * Track the viewport width against {@link defaultBreakpoints}, hydration-safely.
 *
 * @param options SSR width, timing, and window capability.
 * @returns Reactive width, active breakpoint, and comparison helpers.
 */
export function useBreakpoint(
  options?: UseBreakpointOptions,
): BreakpointState<keyof typeof defaultBreakpoints>;
/**
 * Track the viewport width against named breakpoints, hydration-safely.
 *
 * The width starts at `ssrWidth` (or `null`) on both server and client, so
 * the first client render matches the server; the real width is read after
 * mount and on every `resize`. Must run in component setup (or an active
 * effect scope with `immediate: true`); the listener is removed with the scope.
 *
 * @param breakpoints Named minimum widths.
 * @param options SSR width, timing, and window capability.
 * @returns Reactive width, active breakpoint, and comparison helpers.
 */
export function useBreakpoint<const Name extends string>(
  breakpoints: MaybeRefOrGetter<BreakpointMap<Name>>,
  options?: UseBreakpointOptions,
): BreakpointState<Name>;
export function useBreakpoint(
  first?: MaybeRefOrGetter<BreakpointMap> | UseBreakpointOptions,
  second: UseBreakpointOptions = {},
): BreakpointState<string> | BreakpointState<keyof typeof defaultBreakpoints> {
  const source: MaybeRefOrGetter<BreakpointMap> =
    first === undefined || isOptions(first) ? defaultBreakpoints : first;
  const options: UseBreakpointOptions = first !== undefined && isOptions(first) ? first : second;
  if (getCurrentScope() === undefined) {
    throw new Error(`${setupDiagnostic}: call inside component setup or an active effect scope`);
  }
  const width = shallowRef<number | null>(toValue(options.ssrWidth) ?? null);
  let host: BreakpointHost | null = null;
  const update = (): void => {
    if (host) width.value = host.innerWidth;
  };
  const start = (): void => {
    host = options.host === undefined ? browserHost() : options.host;
    if (!host) return;
    update();
    host.addEventListener("resize", update, { passive: true });
  };
  onScopeDispose(() => host?.removeEventListener("resize", update));
  if (options.immediate === true) start();
  else onMounted(start);

  const minimum = (name: string): number => toValue(source)[name] ?? Number.POSITIVE_INFINITY;
  const state: BreakpointState<string> = {
    width,
    active: computed(() => resolveActiveBreakpoint(toValue(source), width.value)),
    isAbove: (name) => width.value !== null && width.value >= minimum(name),
    isBelow: (name) => width.value !== null && width.value < minimum(name),
    isBetween: (lower, upper) =>
      width.value !== null && width.value >= minimum(lower) && width.value < minimum(upper),
  };
  return state;
}

function isOptions(
  value: MaybeRefOrGetter<BreakpointMap> | UseBreakpointOptions,
): value is UseBreakpointOptions {
  if (typeof value === "function" || isRef(value)) return false;
  const keys = Object.keys(value);
  return (
    keys.length === 0 ||
    keys.some((key) => key === "ssrWidth" || key === "immediate" || key === "host") ||
    !Object.values(value).every((entry) => typeof entry === "number")
  );
}

function isName<Name extends string>(breakpoints: BreakpointMap<Name>, key: string): key is Name {
  return Object.hasOwn(breakpoints, key);
}

function browserHost(): BreakpointHost | null {
  return typeof window === "undefined" ? null : window;
}
