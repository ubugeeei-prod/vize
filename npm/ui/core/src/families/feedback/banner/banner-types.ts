import type { PrimitiveAs, PrimitiveElement } from "../../../primitive.ts";

/** Persistent page or section banner role. */
export type BannerRole = "alert" | "region" | "status";

/** Consumer styling tones mirrored by {@link Banner} through `data-tone`. */
export type BannerTone = "accent" | "danger" | "info" | "neutral" | "success" | "warning";

/** Open state mirrored by {@link Banner} through `data-state`. */
export type BannerState = "closed" | "open";

/** Live-region politeness resolved from the selected banner role. */
export type BannerLive = "assertive" | "off" | "polite";

/** Resolved accessibility quality for the rendered banner host. */
export type BannerAriaState = "live" | "named" | "unnamed";

/** Rendered value exposed by {@link Banner}. */
export type BannerElement = PrimitiveElement;

interface BannerBaseProps {
  /** Native element, custom element, or component to render. */
  readonly as?: PrimitiveAs;

  /** Optional root id used as the base for deterministic title and description ids. */
  readonly id?: string;

  /** Visible title rendered into the deterministic title node. */
  readonly title?: string;

  /** Visible description rendered into the deterministic description node. */
  readonly description?: string;

  /** Persistent banner role. `region` requires a name; live roles may announce changes. */
  readonly role?: BannerRole;

  /** Consumer styling tone mirrored to `data-tone`; no CSS is emitted. */
  readonly tone?: BannerTone;

  /** Controlled visibility. `false` hides the banner and suppresses ARIA semantics. */
  readonly open?: boolean;

  /** Render a native dismiss control that requests `open=false`; local state is never mutated. */
  readonly dismissible?: boolean;

  /** Accessible label for the rendered dismiss control. */
  readonly dismissLabel?: string;

  /** Whether live-role announcements should be atomic. */
  readonly atomic?: boolean;

  /** Accessible name when no visible title or `aria-labelledby` supplies one. */
  readonly ariaLabel?: string;

  /** Space-separated ids that label the banner. */
  readonly ariaLabelledby?: string;

  /** Space-separated ids that describe the banner. */
  readonly ariaDescribedby?: string;
}

type BannerNamedByTitle = BannerBaseProps & {
  readonly title: string;
};

type BannerNamedByLabel = BannerBaseProps & {
  readonly ariaLabel: string;
};

type BannerNamedByLabelledby = BannerBaseProps & {
  readonly ariaLabelledby: string;
};

type BannerNamedProps = BannerNamedByTitle | BannerNamedByLabel | BannerNamedByLabelledby;

type BannerLiveRoleProps = BannerBaseProps & {
  readonly role: Exclude<BannerRole, "region">;
};

/**
 * Public props accepted by the Banner primitive.
 *
 * The default `region` role needs an accessible name. TypeScript consumers
 * provide that through `title`, `ariaLabel`, or `ariaLabelledby`; runtime
 * slot-only titles are normalized by the SFC. Live-role `status` and `alert`
 * banners may rely on their contents instead.
 */
export type BannerProps = BannerNamedProps | BannerLiveRoleProps;

/** Emits produced by the Banner primitive. */
export interface BannerEmits {
  /** Request that the controlled `open` prop moves to a new value. */
  "update:open": [open: boolean];

  /** Fired after the dismiss control is activated. */
  dismiss: [nativeEvent: MouseEvent];
}

/** State exposed to every Banner slot. */
export interface BannerSlotState {
  /** Open state mirrored to `data-state`. */
  readonly state: BannerState;

  /** Persistent banner role selected by props. */
  readonly role: BannerRole;

  /** Consumer styling tone mirrored to `data-tone`. */
  readonly tone: BannerTone;

  /** Resolved live-region politeness. */
  readonly live: BannerLive;

  /** Whether the rendered host has a valid accessible name. */
  readonly named: boolean;

  /** Resolved accessibility quality mirrored to `data-aria-state`. */
  readonly ariaState: BannerAriaState;

  /** Deterministic id for the rendered title part. */
  readonly titleId: string;

  /** Deterministic id for the rendered description part. */
  readonly descriptionId: string;

  /** Resolved `aria-labelledby` value, if any. */
  readonly ariaLabelledby: string | undefined;

  /** Resolved `aria-describedby` value, if any. */
  readonly ariaDescribedby: string | undefined;

  /** Whether the controlled dismiss affordance is rendered. */
  readonly dismissible: boolean;
}

/** Slots accepted by the Banner primitive. */
export interface BannerSlots {
  /** Render the primary banner body. */
  default?(props: BannerSlotState): unknown;

  /** Render a consumer-owned title inside the deterministic title node. */
  title?(props: BannerSlotState): unknown;

  /** Render a consumer-owned description inside the deterministic description node. */
  description?(props: BannerSlotState): unknown;

  /** Render trailing actions. */
  actions?(props: BannerSlotState): unknown;
}

/** Public component instance state exposed by the Banner primitive. */
export interface BannerExpose extends BannerSlotState {
  /** Rendered host element or component instance. */
  readonly element: BannerElement | null;

  /** Programmatically focus the rendered banner host, when present and focusable. */
  readonly focus: (options?: FocusOptions) => void;

  /** Request controlled dismissal without changing local state. */
  readonly dismiss: (nativeEvent?: MouseEvent) => void;
}
