import type {
  MotionDelayToken,
  MotionDurationToken,
  MotionEasingToken,
  MotionRecipeHook,
  MotionTokenName,
  MotionTokenOverrides,
} from "./motion-types.ts";

const invalidTokenDiagnostic = "VIZE_UI_MOTION_TOKEN";

/**
 * Duration tokens mirrored from `motion.css`.
 *
 * The packaged stylesheet is the source of truth; these mirrors exist so
 * JavaScript consumers can read the same scale the CSS ships with.
 */
export const motionDurations: Readonly<Record<MotionDurationToken, string>> = Object.freeze({
  instant: "0ms",
  fast: "120ms",
  base: "200ms",
  slow: "320ms",
  deliberate: "480ms",
});

/** Delay tokens mirrored from `motion.css`. */
export const motionDelays: Readonly<Record<MotionDelayToken, string>> = Object.freeze({
  none: "0ms",
  hint: "40ms",
  stagger: "24ms",
});

/** Named easing curves mirrored from `motion.css`. */
export const motionEasings: Readonly<Record<MotionEasingToken, string>> = Object.freeze({
  standard: "cubic-bezier(0.2, 0, 0, 1)",
  decelerate: "cubic-bezier(0.1, 0, 0, 1)",
  accelerate: "cubic-bezier(0.45, 0, 1, 1)",
  emphasized: "cubic-bezier(0.34, 1.3, 0.32, 1)",
  linear: "linear",
});

const recipeHooks: readonly MotionRecipeHook[] = [
  "enter-duration",
  "enter-easing",
  "exit-duration",
  "exit-easing",
  "move-duration",
  "move-easing",
  "emphasis-duration",
  "emphasis-easing",
  "slide-distance",
  "scale-from",
];

const tokenNames: ReadonlySet<string> = new Set<string>([
  ...Object.keys(motionDurations).map((token) => `duration-${token}`),
  ...Object.keys(motionDelays).map((token) => `delay-${token}`),
  ...Object.keys(motionEasings).map((token) => `ease-${token}`),
  ...recipeHooks,
]);

function assertTokenName(name: string): void {
  if (!tokenNames.has(name)) {
    throw new TypeError(`${invalidTokenDiagnostic}: unknown motion token "${name}"`);
  }
}

/** Custom-property name (`--vize-ui-motion-*`) for one motion token. */
export function motionTokenProperty(name: MotionTokenName): string {
  assertTokenName(name);
  return `--vize-ui-motion-${name}`;
}

/** `var()` reference to one motion token for imperative style composition. */
export function motionTokenVar(name: MotionTokenName): string {
  return `var(${motionTokenProperty(name)})`;
}

/**
 * Override motion tokens on one element's subtree.
 *
 * Custom properties inherit, so scoping an override to an element retunes
 * every recipe below it — replacing a curve without forking a component.
 * Returns a restore function that reinstates the element's previous values.
 */
export function setMotionTokens(
  element: ElementCSSInlineStyle,
  overrides: MotionTokenOverrides,
): () => void {
  if (typeof element?.style?.setProperty !== "function") {
    throw new TypeError(`${invalidTokenDiagnostic}: overrides need an element with inline style`);
  }

  const previous = new Map<string, string>();
  for (const [name, value] of Object.entries(overrides)) {
    if (value === undefined) continue;
    if (typeof value !== "string" || value.trim().length === 0) {
      throw new TypeError(`${invalidTokenDiagnostic}: token "${name}" needs a non-empty string`);
    }
    const property = motionTokenProperty(name as MotionTokenName);
    if (!previous.has(property)) previous.set(property, element.style.getPropertyValue(property));
    element.style.setProperty(property, value);
  }

  return () => {
    for (const [property, value] of previous) {
      if (value === "") element.style.removeProperty(property);
      else element.style.setProperty(property, value);
    }
    previous.clear();
  };
}
