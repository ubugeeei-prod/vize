import type {
  EmojiPickerCategory,
  EmojiPickerFilter,
  EmojiPickerSection,
  EmojiSkinTone,
} from "./emoji-picker-model.ts";

export type {
  EmojiCellLocation,
  EmojiPickerAccessors,
  EmojiPickerCategory,
  EmojiPickerFilter,
  EmojiPickerSection,
  EmojiSkinTone,
  EmojiVirtualGrid,
} from "./emoji-picker-model.ts";

/** Public props accepted by `EmojiPickerRoot`. */
export interface EmojiPickerRootProps<T> {
  /**
   * Consumer-owned base id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /** Every emoji item, in display order. No dataset ships with the picker. @default required */
  readonly items: readonly T[];

  /** Native glyph for the default tone. @default required */
  readonly getEmoji: (item: T) => string;

  /** Human-readable name used for search, labels, and the preview. @default required */
  readonly getName: (item: T) => string;

  /**
   * Extra search keywords.
   *
   * @default undefined
   */
  readonly getKeywords?: (item: T) => readonly string[];

  /**
   * Category id of an item; omit for a single uncategorized section.
   *
   * @default undefined
   */
  readonly getCategory?: (item: T) => string;

  /**
   * Skin-tone variants indexed 0–5 (0 is the default tone).
   *
   * @default undefined
   */
  readonly getSkins?: (item: T) => readonly string[] | undefined;

  /**
   * Category order and labels; unknown categories follow in first-seen order.
   *
   * @default undefined
   */
  readonly categories?: readonly EmojiPickerCategory[];

  /**
   * Cells per row; must match the consumer's CSS grid.
   *
   * @default 8
   */
  readonly columns?: number;

  /**
   * Controlled skin tone. `undefined` selects uncontrolled behavior.
   *
   * @default undefined
   */
  readonly skinTone?: EmojiSkinTone;

  /**
   * Initial skin tone for uncontrolled use.
   *
   * @default 0
   */
  readonly defaultSkinTone?: EmojiSkinTone;

  /**
   * Controlled search text. `undefined` selects uncontrolled behavior.
   *
   * @default undefined
   */
  readonly search?: string;

  /**
   * Search filter. Defaults to an accent- and case-insensitive name/keyword match.
   *
   * @default undefined
   */
  readonly filter?: EmojiPickerFilter<T>;

  /**
   * Consumer-owned recently used items, shown as the first section.
   *
   * @default undefined
   */
  readonly recent?: readonly T[];

  /**
   * Label of the recent section.
   *
   * @default "Recently used"
   */
  readonly recentLabel?: string;

  /**
   * Label of the search results section.
   *
   * @default "Search results"
   */
  readonly searchLabel?: string;

  /**
   * Label of the single section used when `getCategory` is omitted.
   *
   * @default "Emoji"
   */
  readonly defaultLabel?: string;

  /**
   * Reading direction; `rtl` mirrors ArrowLeft and ArrowRight.
   *
   * @default "ltr"
   */
  readonly dir?: "ltr" | "rtl";
}

/** State exposed to the root slot. */
export interface EmojiPickerSlotState<T> {
  /** Visible sections: render one `EmojiPickerCategory` per entry. */
  readonly sections: readonly EmojiPickerSection<T>[];

  /** Highlighted item, if any. */
  readonly activeItem: T | undefined;

  /** Current skin tone. */
  readonly skinTone: EmojiSkinTone;

  /** Current search text. */
  readonly search: string;

  /** Whether no item is visible. */
  readonly empty: boolean;

  /** Glyph of an item with the current skin tone applied. */
  readonly glyphOf: (item: T) => string;
}

/** State exposed to `EmojiPickerCategory` cell slots. */
export interface EmojiPickerCellSlotState<T> {
  /** Item rendered in this cell. */
  readonly item: T;

  /** Item index inside the section. */
  readonly index: number;

  /** Column of the cell. */
  readonly column: number;
}

/** State exposed to `EmojiPickerItem` slots. */
export interface EmojiPickerItemSlotState<T> {
  /** Item value. */
  readonly item: T;

  /** Glyph with the skin tone applied. */
  readonly glyph: string;

  /** Item name. */
  readonly name: string;

  /** Whether the cell is highlighted. */
  readonly active: boolean;
}

/** State exposed to `EmojiPickerPreview` slots. */
export interface EmojiPickerPreviewSlotState {
  /** Highlighted item, or `undefined`. */
  readonly item: unknown;

  /** Glyph with the skin tone applied, or an empty string. */
  readonly glyph: string;

  /** Item name, or an empty string. */
  readonly name: string;
}

/** Public instance exposed by `EmojiPickerRoot`. */
export interface EmojiPickerRootExpose<T> {
  /** Highlighted item. */
  readonly activeItem: T | undefined;

  /** Current skin tone. */
  readonly skinTone: EmojiSkinTone;

  /** Move DOM focus into the grid. */
  readonly focusGrid: () => void;

  /** Replace the search text. */
  readonly setSearch: (text: string) => void;
}
