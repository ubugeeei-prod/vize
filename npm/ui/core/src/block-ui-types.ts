import type { PrimitiveElement } from "./primitive.ts";

/** Stable blocking state mirrored by {@link BlockUI} through `data-state`. */
export type BlockUIState = "blocked" | "idle";

/** Consumer styling and status reason mirrored by {@link BlockUI} through `data-reason`. */
export type BlockUIReason = "loading" | "saving" | "syncing" | "stale" | "offline";

/** Interaction policy mirrored by {@link BlockUI} through `data-interaction`. */
export type BlockUIInteraction = "none" | "inert";

/** Optional announcement policy mirrored by {@link BlockUI} through `data-announcement`. */
export type BlockUIAnnouncement = "off" | "polite" | "assertive";

/** Rendered value exposed by {@link BlockUI}. */
export type BlockUIElement = PrimitiveElement;

/** State exposed to the default BlockUI slot. */
export interface BlockUISlotState {
  /** Whether the region currently represents blocked work. */
  readonly blocked: boolean;

  /** Stable state token mirrored to `data-state`. */
  readonly state: BlockUIState;

  /** Blocking reason mirrored to `data-reason`. */
  readonly reason: BlockUIReason;

  /** Interaction policy mirrored to `data-interaction`. */
  readonly interaction: BlockUIInteraction;

  /** Announcement policy mirrored to `data-announcement`. */
  readonly announcement: BlockUIAnnouncement;
}

/** Public component instance state exposed by the BlockUI primitive. */
export interface BlockUIExpose extends BlockUISlotState {
  /** Rendered host element or component instance. */
  readonly element: BlockUIElement | null;
}
