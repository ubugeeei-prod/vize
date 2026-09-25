/** Open state mirrored through `data-state`. */
export type LightboxState = "closed" | "open";

/** Reading direction used for horizontal keys and swipes. */
export type LightboxDirection = "ltr" | "rtl";

/** Why the current item changed. */
export type LightboxChangeReason =
  | "api"
  | "keyboard"
  | "next"
  | "previous"
  | "swipe"
  | "thumbnail"
  | "trigger";

/** Localizable strings used by the lightbox parts. */
export interface LightboxMessages {
  /** Accessible name of the dialog. */
  readonly dialog: string;

  /** Role description of the current item, announced with its position. */
  readonly slide: string;

  /** Accessible name of the previous button. */
  readonly previous: string;

  /** Accessible name of the next button. */
  readonly next: string;

  /** Accessible name of the close button. */
  readonly close: string;

  /** Accessible name of the thumbnail tablist. */
  readonly thumbnails: string;

  /** Position text, e.g. "3 of 10". Also labels the current item. */
  readonly counter: (position: number, count: number) => string;

  /** Accessible name of one thumbnail, e.g. "Show item 3 of 10". */
  readonly thumbnail: (position: number, count: number) => string;
}

/** Partial overrides accepted by the `messages` prop. */
export type LightboxMessageOverrides = Partial<LightboxMessages>;

/** Default English messages. */
export const defaultLightboxMessages: LightboxMessages = Object.freeze({
  close: "Close",
  counter: (position: number, count: number) => `${position} of ${count}`,
  dialog: "Media viewer",
  next: "Next item",
  previous: "Previous item",
  slide: "slide",
  thumbnail: (position: number, count: number) => `Show item ${position} of ${count}`,
  thumbnails: "Choose item",
});

/** State exposed to every lightbox part. */
export interface LightboxPartSlotState {
  /** Whether the viewer is open. */
  readonly open: boolean;

  /** Zero-based current item index. */
  readonly index: number;

  /** Number of items. */
  readonly count: number;

  /** Whether previous navigation is available. */
  readonly canGoPrevious: boolean;

  /** Whether next navigation is available. */
  readonly canGoNext: boolean;

  /** Stable state token. */
  readonly state: LightboxState;
}

/** State exposed to the LightboxRoot default slot, with the inferred item type. */
export interface LightboxSlotState<Item> extends LightboxPartSlotState {
  /** Current item, or `undefined` without items. */
  readonly item: Item | undefined;

  /** Every item. */
  readonly items: readonly Item[];
}

/** State exposed to LightboxTrigger and LightboxThumbnail slots. */
export interface LightboxIndexSlotState {
  /** Zero-based item index the control targets. */
  readonly index: number;

  /** Whether that item is current. */
  readonly current: boolean;
}

/** State exposed to the LightboxCounter slot. */
export interface LightboxCounterSlotState {
  /** One-based current position. */
  readonly position: number;

  /** Number of items. */
  readonly count: number;

  /** Localized position text. */
  readonly text: string;
}

/** Public instance exposed by LightboxRoot. */
export interface LightboxRootExpose<Item> extends LightboxSlotState<Item> {
  /** Open at an index. Reports whether open or index changed. */
  readonly openAt: (index: number) => boolean;

  /** Close the viewer. Reports whether it was open. */
  readonly close: () => boolean;

  /** Show one item. Reports whether the index changed. */
  readonly goTo: (index: number) => boolean;

  /** Show the next item (wrapping with `loop`). */
  readonly next: () => boolean;

  /** Show the previous item (wrapping with `loop`). */
  readonly previous: () => boolean;
}

/** Public instance exposed by LightboxContent. */
export interface LightboxContentExpose {
  /** Rendered stage element inside the dialog, while open. */
  readonly element: HTMLDivElement | null;
}

/** Public instance exposed by LightboxItem. */
export interface LightboxItemExpose {
  /** Rendered item element. */
  readonly element: HTMLDivElement | null;

  /** Deterministic id referenced by thumbnails. */
  readonly id: string;
}

/** Public instance exposed by lightbox buttons. */
export interface LightboxButtonExpose {
  /** Rendered native button. */
  readonly element: HTMLButtonElement | null;

  /** Whether the button is disabled. */
  readonly disabled: boolean;
}

/** Public instance exposed by LightboxCounter and LightboxThumbnails. */
export interface LightboxElementExpose {
  /** Rendered element. */
  readonly element: HTMLDivElement | null;
}
