import type { BannerAriaState, BannerLive, BannerRole } from "./banner-types.ts";

/** Input accepted by {@link normalizeBannerAria}. */
export interface BannerAriaInput {
  /** Persistent banner role selected by props. */
  readonly role: BannerRole;

  /** Whether the title part exists and can name the banner. */
  readonly hasTitle: boolean;

  /** Deterministic id for the title part. */
  readonly titleId: string;

  /** Whether the description part exists and can describe the banner. */
  readonly hasDescription: boolean;

  /** Deterministic id for the description part. */
  readonly descriptionId: string;

  /** Consumer-owned accessible label. */
  readonly ariaLabel?: string | undefined;

  /** Consumer-owned accessible labelledby references. */
  readonly ariaLabelledby?: string | undefined;

  /** Consumer-owned accessible describedby references. */
  readonly ariaDescribedby?: string | undefined;

  /** Whether live announcements should be atomic. */
  readonly atomic: boolean;
}

/** Normalized ARIA attributes for the rendered Banner host. */
export interface NormalizedBannerAria {
  readonly role: BannerRole | undefined;
  readonly ariaState: BannerAriaState;
  readonly live: BannerLive;
  readonly named: boolean;
  readonly ariaLabel: string | undefined;
  readonly ariaLabelledby: string | undefined;
  readonly ariaDescribedby: string | undefined;
  readonly ariaLive: "assertive" | "polite" | undefined;
  readonly ariaAtomic: "false" | "true" | undefined;
}

/** Resolve accessible naming, description, and live-region attributes for Banner. */
export function normalizeBannerAria(input: BannerAriaInput): NormalizedBannerAria {
  const explicitLabelledby = normalizeIdRefs(input.ariaLabelledby);
  const ariaLabel = explicitLabelledby === undefined ? normalizeText(input.ariaLabel) : undefined;
  const ariaLabelledby =
    explicitLabelledby ?? (ariaLabel === undefined && input.hasTitle ? input.titleId : undefined);
  const ariaDescribedby = joinIdRefs(
    normalizeIdRefs(input.ariaDescribedby),
    input.hasDescription ? input.descriptionId : undefined,
  );
  const named = ariaLabel !== undefined || ariaLabelledby !== undefined;
  const live = bannerLiveForRole(input.role);
  const liveRole = input.role === "alert" || input.role === "status";
  const role = liveRole || named ? input.role : undefined;
  const ariaState: BannerAriaState = liveRole ? "live" : named ? "named" : "unnamed";

  return {
    ariaAtomic: liveRole ? (input.atomic ? "true" : "false") : undefined,
    ariaDescribedby,
    ariaLabel,
    ariaLabelledby,
    ariaLive: live === "off" ? undefined : live,
    ariaState,
    live,
    named,
    role,
  };
}

function bannerLiveForRole(role: BannerRole): BannerLive {
  if (role === "alert") return "assertive";
  if (role === "status") return "polite";
  return "off";
}

function normalizeText(value: string | undefined): string | undefined {
  if (value === undefined) return undefined;
  const normalized = value.replaceAll(/\s+/g, " ").trim();
  return normalized.length === 0 ? undefined : normalized;
}

function normalizeIdRefs(value: string | undefined): string | undefined {
  return normalizeText(value);
}

function joinIdRefs(...values: readonly (string | undefined)[]): string | undefined {
  const parts = values.filter((value): value is string => value !== undefined);
  return parts.length === 0 ? undefined : parts.join(" ");
}
