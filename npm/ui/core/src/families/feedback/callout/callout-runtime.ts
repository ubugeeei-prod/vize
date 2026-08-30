import type { CalloutAriaState, CalloutLive, CalloutRole } from "./callout-types.ts";

interface CalloutAriaStateOptions {
  readonly ariaHidden?: boolean | undefined;
  readonly role: CalloutRole;
}

/** Normalize an ARIA IDREF list without accepting an empty rendered attribute. */
export function normalizeCalloutIdReferenceList(value: string | undefined): string | undefined {
  const normalized = value?.trim().replaceAll(/\s+/g, " ");
  return normalized === "" ? undefined : normalized;
}

/** Normalize a direct accessible label while preserving intentional inner spacing. */
export function normalizeCalloutLabel(value: string | undefined): string | undefined {
  const normalized = value?.trim();
  return normalized === "" ? undefined : normalized;
}

/** Resolve whether the Callout participates in the accessibility tree. */
export function resolveCalloutAriaState(options: CalloutAriaStateOptions): CalloutAriaState {
  return options.ariaHidden === true ? "decorative" : options.role;
}

/** Derive live-region politeness from the resolved accessibility state. */
export function resolveCalloutLive(ariaState: CalloutAriaState): CalloutLive | undefined {
  if (ariaState === "alert") return "assertive";
  if (ariaState === "status") return "polite";
  return undefined;
}
