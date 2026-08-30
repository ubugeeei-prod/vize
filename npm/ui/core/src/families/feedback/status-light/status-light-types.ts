import type { PrimitiveAs, PrimitiveElement } from "../../../primitive.ts";

/** Presence or health state mirrored by {@link StatusLight} through `data-state`. */
export type StatusLightState = "away" | "busy" | "offline" | "online" | "unknown";

/** Consumer styling tones mirrored by {@link StatusLight} through `data-tone`. */
export type StatusLightTone = "accent" | "danger" | "info" | "neutral" | "success" | "warning";

/** Consumer size tokens mirrored by {@link StatusLight} through `data-size`. */
export type StatusLightSize = "sm" | "md" | "lg";

/** Accessibility role used when the status light is not decorative. */
export type StatusLightRole = "img" | "status";

/** Resolved accessibility semantics exposed by {@link StatusLight}. */
export type StatusLightAriaState = "decorative" | StatusLightRole;

/** Rendered value exposed by {@link StatusLight}. */
export type StatusLightElement = PrimitiveElement;

/** Public props accepted by the StatusLight primitive. */
export interface StatusLightProps {
  /** Native element, custom element, or component to render. */
  readonly as?: PrimitiveAs;

  /** Presence or health state mirrored to `data-state`. */
  readonly state?: StatusLightState;

  /** Styling tone mirrored to `data-tone`; no CSS is emitted. */
  readonly tone?: StatusLightTone;

  /** Consumer size token mirrored to `data-size`; no CSS is emitted. */
  readonly size?: StatusLightSize;

  /** Accessibility role used when the light is not decorative. */
  readonly role?: StatusLightRole;

  /** Whether status announcements should be atomic when `role="status"`. */
  readonly atomic?: boolean;

  /** Hide the light from assistive technology. Unlabelled lights are decorative by default. */
  readonly ariaHidden?: boolean;

  /** Accessible name when no visible label or `aria-labelledby` supplies one. */
  readonly ariaLabel?: string;

  /** Space-separated ids that label the status light. */
  readonly ariaLabelledby?: string;

  /** Space-separated ids that describe the status light. */
  readonly ariaDescribedby?: string;
}

/** State exposed to the default StatusLight slot. */
export interface StatusLightSlotState {
  /** Presence or health state mirrored to `data-state`. */
  readonly state: StatusLightState;

  /** Consumer styling tone mirrored to `data-tone`. */
  readonly tone: StatusLightTone;

  /** Consumer size token mirrored to `data-size`. */
  readonly size: StatusLightSize;

  /** Whether accessibility semantics are decorative, image-like, or status-like. */
  readonly ariaState: StatusLightAriaState;

  /** Whether the rendered host is hidden from assistive technology. */
  readonly decorative: boolean;
}

/** Public component instance state exposed by the StatusLight primitive. */
export interface StatusLightExpose extends StatusLightSlotState {
  /** Rendered host element or component instance. */
  readonly element: StatusLightElement | null;
}
