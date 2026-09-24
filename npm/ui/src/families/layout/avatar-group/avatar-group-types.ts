/** Localizable strings used by {@link AvatarGroup} and {@link AvatarGroupOverflow}. */
export interface AvatarGroupMessages {
  /** Accessible name of the overflow tile, e.g. "3 more". */
  readonly overflow: (count: number) => string;

  /** Visible overflow text, e.g. "+3". */
  readonly overflowText: (count: number) => string;
}

/** Partial overrides accepted by the `messages` props. */
export type AvatarGroupMessageOverrides = Partial<AvatarGroupMessages>;

/** Default English messages. */
export const defaultAvatarGroupMessages: AvatarGroupMessages = Object.freeze({
  overflow: (count: number) => `${count} more`,
  overflowText: (count: number) => `+${count}`,
});

/** State exposed by AvatarGroup through `data-state`. */
export type AvatarGroupState = "collapsed" | "empty" | "expanded";

/** Slot state for one visible item. */
export interface AvatarGroupItemSlotState<Item> {
  /** Consumer item. */
  readonly item: Item;

  /** Zero-based item index in `items`. */
  readonly index: number;
}

/** Slot state for the overflow tile. */
export interface AvatarGroupOverflowSlotState<Item> {
  /** Items hidden behind the overflow tile, in order. Excludes the server-only `total` remainder. */
  readonly hiddenItems: readonly Item[];

  /** Number of hidden people, including any `total` remainder not present in `items`. */
  readonly count: number;

  /** Accessible overflow name from the messages. */
  readonly label: string;

  /** Visible overflow text from the messages. */
  readonly text: string;
}

/** State exposed to the AvatarGroup default slot. */
export interface AvatarGroupSlotState<Item> {
  /** Items rendered as tiles. */
  readonly visibleItems: readonly Item[];

  /** Items collapsed into the overflow tile. */
  readonly hiddenItems: readonly Item[];

  /** Hidden count including any `total` remainder. */
  readonly overflowCount: number;

  /** Stable state token. */
  readonly state: AvatarGroupState;
}

/** Public instance exposed by AvatarGroup. */
export interface AvatarGroupExpose<Item> extends AvatarGroupSlotState<Item> {
  /** Rendered list element. */
  readonly element: HTMLUListElement | null;
}

/** Public instance exposed by AvatarGroupOverflow. */
export interface AvatarGroupOverflowExpose {
  /** Rendered overflow element. */
  readonly element: HTMLSpanElement | null;

  /** Hidden count announced by the tile. */
  readonly count: number;
}
