import type { SurfaceAriaState } from "./surface-types.ts";

interface SurfaceAriaOptions {
  readonly ariaDescribedby?: string | undefined;
  readonly ariaLabelledby?: string | undefined;
}

/** Normalize one ARIA IDREF list without inventing ids or request-global state. */
export function normalizeSurfaceIdReference(value: string | undefined): string | undefined {
  const normalized = value?.trim().replaceAll(/\s+/g, " ");
  return normalized === "" ? undefined : normalized;
}

/** Resolve Surface accessibility props into native ARIA attributes. */
export function resolveSurfaceAria(options: SurfaceAriaOptions): SurfaceAriaState {
  return {
    ariaDescribedby: normalizeSurfaceIdReference(options.ariaDescribedby),
    ariaLabelledby: normalizeSurfaceIdReference(options.ariaLabelledby),
  };
}
