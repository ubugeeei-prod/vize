import type { PrimitiveAs, PrimitiveElement } from "../../../primitive.ts";

/** ARIA role used by the Callout root when it is not hidden from assistive technology. */
export type CalloutRole = "alert" | "note" | "status";

/** Resolved accessibility state mirrored by {@link Callout} through `data-aria-state`. */
export type CalloutAriaState = "decorative" | CalloutRole;

/** Consumer styling tones mirrored by {@link Callout} through `data-tone`. */
export type CalloutTone = "accent" | "danger" | "info" | "neutral" | "success" | "warning";

/** Consumer spacing density mirrored by {@link Callout} through `data-density`. */
export type CalloutDensity = "compact" | "comfortable";

/** Visibility state mirrored by {@link Callout} through `data-state`. */
export type CalloutState = "closed" | "open";

/** Explicit live-region politeness derived from the selected Callout role. */
export type CalloutLive = "assertive" | "polite";

/** Rendered value exposed by {@link Callout}. */
export type CalloutElement = PrimitiveElement;

/** Public props accepted by the Callout primitive. */
export interface CalloutProps {
  /**
   * Native element, custom element, or component to render.
   *
   * @default "section"
   */
  readonly as?: PrimitiveAs;

  /**
   * Consumer-owned root id for anchors or application state.
   *
   * @default undefined
   */
  readonly id?: string;

  /**
   * Accessibility role. `note` is static, `status` is polite, and `alert` is assertive.
   *
   * @default "note"
   */
  readonly role?: CalloutRole;

  /**
   * Whether the Callout is visible.
   *
   * @default true
   */
  readonly open?: boolean;

  /**
   * Whether assistive technology should present the whole live region on updates.
   *
   * @default true
   */
  readonly atomic?: boolean;

  /**
   * Styling tone mirrored to `data-tone`; no CSS is emitted.
   *
   * @default "neutral"
   */
  readonly tone?: CalloutTone;

  /**
   * Spacing density mirrored to `data-density`; no CSS is emitted.
   *
   * @default "comfortable"
   */
  readonly density?: CalloutDensity;

  /**
   * Whether the optional icon wrapper is hidden from assistive technology.
   *
   * @default true
   */
  readonly iconAriaHidden?: boolean;

  /**
   * Hide the entire Callout from assistive technology.
   *
   * @default undefined
   */
  readonly ariaHidden?: boolean;

  /**
   * Accessible name when no visible title or `aria-labelledby` supplies one.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids that label the Callout.
   *
   * @default generated from the title slot when present
   */
  readonly ariaLabelledby?: string;

  /**
   * Space-separated ids that describe the Callout.
   *
   * @default generated from the description slot when present
   */
  readonly ariaDescribedby?: string;

  /**
   * Consumer-owned id for the title slot wrapper.
   *
   * @default generated when the title slot is present
   */
  readonly titleId?: string;

  /**
   * Consumer-owned id for the description slot wrapper.
   *
   * @default generated when the description slot is present
   */
  readonly descriptionId?: string;
}

/** State exposed to every Callout slot. */
export interface CalloutSlotState {
  /** Whether the Callout is visible. */
  readonly open: boolean;

  /** Visibility state mirrored to `data-state`. */
  readonly state: CalloutState;

  /** Requested accessibility role. */
  readonly role: CalloutRole;

  /** Resolved accessibility state after `ariaHidden` is applied. */
  readonly ariaState: CalloutAriaState;

  /** Derived live-region politeness, when `role` is `alert` or `status`. */
  readonly live: CalloutLive | undefined;

  /** Whether assistive technology should present the whole live region on updates. */
  readonly atomic: boolean;

  /** Consumer styling tone mirrored to `data-tone`. */
  readonly tone: CalloutTone;

  /** Consumer spacing density mirrored to `data-density`. */
  readonly density: CalloutDensity;

  /** Resolved id for the title wrapper. */
  readonly titleId: string | undefined;

  /** Resolved id for the description wrapper. */
  readonly descriptionId: string | undefined;

  /** Resolved `aria-labelledby` value for the root. */
  readonly ariaLabelledby: string | undefined;

  /** Resolved `aria-describedby` value for the root. */
  readonly ariaDescribedby: string | undefined;

  /** Whether the icon slot is rendered. */
  readonly hasIcon: boolean;

  /** Whether the title slot is rendered. */
  readonly hasTitle: boolean;

  /** Whether the description slot is rendered. */
  readonly hasDescription: boolean;

  /** Whether the actions slot is rendered. */
  readonly hasActions: boolean;
}

/** Slots exposed by the Callout component. */
export interface CalloutSlots {
  /** Renders the main Callout body. */
  default(props: CalloutSlotState): unknown;

  /** Renders an optional consumer-owned icon. */
  icon(props: CalloutSlotState): unknown;

  /** Renders an optional accessible title. */
  title(props: CalloutSlotState): unknown;

  /** Renders an optional accessible description. */
  description(props: CalloutSlotState): unknown;

  /** Renders optional interactive or navigational actions. */
  actions(props: CalloutSlotState): unknown;
}

/** Public component instance state exposed by the Callout primitive. */
export interface CalloutExpose extends CalloutSlotState {
  /** Rendered host element or component instance. */
  readonly element: CalloutElement | null;
}
