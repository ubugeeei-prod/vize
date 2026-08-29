import type { ShallowRef } from "vue";

/** ARIA live-region roles accepted by the Alert primitive. */
export type AlertRole = "alert" | "status";

/** Consumer styling variants exposed only through data attributes and slot state. */
export type AlertVariant = "danger" | "info" | "success" | "warning";

/** Visibility state exposed to data attributes and slot consumers. */
export type AlertState = "closed" | "open";

/** State exposed to the default slot for external chrome and dismissal composition. */
export interface AlertSlotState {
  /** Whether the alert is currently visible and announceable. */
  readonly open: boolean;

  /** ARIA role used by the alert root. */
  readonly role: AlertRole;

  /** Visibility state mirrored to `data-state`. */
  readonly state: AlertState;

  /** Styling variant mirrored to `data-variant`. */
  readonly variant: AlertVariant;
}

/** Methods and state exposed by the alert component instance. */
export interface AlertExpose {
  /** Rendered alert root for composition with application-owned effects. */
  readonly element: Readonly<ShallowRef<HTMLDivElement | null>>;
}
