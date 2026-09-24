/**
 * Pure, data-free helpers for EmojiPicker: accessor-driven filtering, section
 * building, skin-tone glyph resolution, and the padded virtual grid used for
 * 2-D navigation. No emoji dataset ships; consumers supply typed items.
 */

/** Skin-tone index: 0 is the default (yellow) tone, 1–5 are Fitzpatrick tones. */
export type EmojiSkinTone = 0 | 1 | 2 | 3 | 4 | 5;

/** One category shown by the picker. */
export interface EmojiPickerCategory {
  /** Stable category id matched against `getCategory(item)`. */
  readonly id: string;

  /** Visible and accessible category label. */
  readonly label: string;
}

/** Accessors mapping consumer items to emoji data. */
export interface EmojiPickerAccessors<T> {
  /** Native glyph for the default tone. */
  readonly getEmoji: (item: T) => string;

  /** Human-readable name used for search, labels, and the preview. */
  readonly getName: (item: T) => string;

  /** Extra search keywords. */
  readonly getKeywords?: ((item: T) => readonly string[]) | undefined;

  /** Category id of the item. */
  readonly getCategory?: ((item: T) => string) | undefined;

  /** Skin-tone variants indexed by {@link EmojiSkinTone}; index 0 is the default tone. */
  readonly getSkins?: ((item: T) => readonly string[] | undefined) | undefined;
}

/** Filter deciding whether an item matches the search text. */
export type EmojiPickerFilter<T> = (
  item: T,
  query: string,
  accessors: EmojiPickerAccessors<T>,
) => boolean;

/** One rendered section: a category, the recent list, or search results. */
export interface EmojiPickerSection<T> {
  /** Stable section id (`recent`, `search`, or a category id). */
  readonly id: string;

  /** Visible section label. */
  readonly label: string;

  /** Items in display order. */
  readonly items: readonly T[];

  /** Items chunked into rows of `columns` cells. */
  readonly rows: readonly (readonly T[])[];
}

/** Options for {@link buildEmojiSections}. */
export interface EmojiSectionOptions<T> {
  readonly items: readonly T[];
  readonly accessors: EmojiPickerAccessors<T>;
  readonly columns: number;
  readonly search: string;
  readonly filter: EmojiPickerFilter<T>;
  readonly categories?: readonly EmojiPickerCategory[] | undefined;
  readonly recent?: readonly T[] | undefined;
  readonly recentLabel: string;
  readonly searchLabel: string;
  readonly defaultLabel: string;
}

/** Normalize text for accent- and case-insensitive matching. */
export function normalizeEmojiText(text: string): string {
  return text
    .normalize("NFKD")
    .replace(/\p{M}+/gu, "")
    .toLocaleLowerCase()
    .trim();
}

/** Default filter: the name or any keyword contains the query. */
export function containsEmojiFilter<T>(
  item: T,
  query: string,
  accessors: EmojiPickerAccessors<T>,
): boolean {
  const needle = normalizeEmojiText(query);
  if (needle.length === 0) return true;
  if (normalizeEmojiText(accessors.getName(item)).includes(needle)) return true;
  return (accessors.getKeywords?.(item) ?? []).some((keyword) =>
    normalizeEmojiText(keyword).includes(needle),
  );
}

/** Clamp any number to a valid skin tone. */
export function toSkinTone(value: number): EmojiSkinTone {
  const tone = Number.isFinite(value) ? Math.min(5, Math.max(0, Math.trunc(value))) : 0;
  switch (tone) {
    case 1:
      return 1;
    case 2:
      return 2;
    case 3:
      return 3;
    case 4:
      return 4;
    case 5:
      return 5;
    default:
      return 0;
  }
}

/** Glyph for `item` with `tone` applied when the item has skin variants. */
export function emojiGlyph<T>(
  item: T,
  tone: EmojiSkinTone,
  accessors: EmojiPickerAccessors<T>,
): string {
  const skins = tone === 0 ? undefined : accessors.getSkins?.(item);
  const variant = skins?.[tone];
  return variant !== undefined && variant.length > 0 ? variant : accessors.getEmoji(item);
}

/** Chunk items into rows of `columns` cells. */
export function chunkEmojiRows<T>(items: readonly T[], columns: number): readonly (readonly T[])[] {
  const size = Math.max(1, Math.floor(columns));
  const rows: (readonly T[])[] = [];
  for (let index = 0; index < items.length; index += size) {
    rows.push(Object.freeze(items.slice(index, index + size)));
  }
  return Object.freeze(rows);
}

function section<T>(id: string, label: string, items: readonly T[], columns: number) {
  return Object.freeze({
    id,
    items: Object.freeze([...items]),
    label,
    rows: chunkEmojiRows(items, columns),
  });
}

/**
 * Build the visible sections. A non-empty search collapses everything into
 * one "search" section; otherwise `recent` comes first, then `categories` in
 * order, then unknown categories in first-seen order. Empty sections are
 * omitted.
 */
export function buildEmojiSections<T>(
  options: EmojiSectionOptions<T>,
): readonly EmojiPickerSection<T>[] {
  const { accessors, columns } = options;
  if (normalizeEmojiText(options.search).length > 0) {
    const matches = options.items.filter((item) => options.filter(item, options.search, accessors));
    return matches.length === 0
      ? Object.freeze([])
      : Object.freeze([section("search", options.searchLabel, matches, columns)]);
  }
  const sections: EmojiPickerSection<T>[] = [];
  if (options.recent !== undefined && options.recent.length > 0) {
    sections.push(section("recent", options.recentLabel, options.recent, columns));
  }
  const getCategory = accessors.getCategory;
  if (getCategory === undefined) {
    if (options.items.length > 0) {
      sections.push(section("all", options.defaultLabel, options.items, columns));
    }
    return Object.freeze(sections);
  }
  const groups = new Map<string, T[]>();
  for (const item of options.items) {
    const id = getCategory(item);
    const group = groups.get(id);
    if (group === undefined) groups.set(id, [item]);
    else group.push(item);
  }
  for (const category of options.categories ?? []) {
    const group = groups.get(category.id);
    if (group === undefined) continue;
    sections.push(section(category.id, category.label, group, columns));
    groups.delete(category.id);
  }
  for (const [id, group] of groups) sections.push(section(id, id, group, columns));
  return Object.freeze(sections);
}

/** Location of one cell inside the sections. */
export interface EmojiCellLocation {
  /** Section position. */
  readonly section: number;

  /** Item index inside the section. */
  readonly index: number;
}

/**
 * Padded virtual grid: each section starts a new row and short rows are
 * padded to `columns`, so a flat index maps to row `floor(i / columns)` and
 * column `i % columns` across every section.
 */
export interface EmojiVirtualGrid {
  /** Total number of virtual cells (rows × columns). */
  readonly count: number;

  /** Number of item rows (label rows excluded). */
  readonly rows: number;

  /** Virtual index of the first row of each section. */
  readonly rowOffsets: readonly number[];

  /** Cell at a virtual index, or `undefined` for padding. */
  readonly cellAt: (virtualIndex: number) => EmojiCellLocation | undefined;

  /** Virtual index of a section item. */
  readonly indexOf: (section: number, index: number) => number;
}

/** Build the padded virtual grid for `sections`. */
export function createEmojiVirtualGrid<T>(
  sections: readonly EmojiPickerSection<T>[],
  columns: number,
): EmojiVirtualGrid {
  const size = Math.max(1, Math.floor(columns));
  const rowOffsets: number[] = [];
  let rows = 0;
  for (const current of sections) {
    rowOffsets.push(rows);
    rows += current.rows.length;
  }
  const indexOf = (sectionIndex: number, index: number): number => {
    const offset = rowOffsets[sectionIndex] ?? 0;
    return (offset + Math.floor(index / size)) * size + (index % size);
  };
  const cellAt = (virtualIndex: number): EmojiCellLocation | undefined => {
    if (virtualIndex < 0) return undefined;
    const row = Math.floor(virtualIndex / size);
    const column = virtualIndex % size;
    for (let sectionIndex = sections.length - 1; sectionIndex >= 0; sectionIndex--) {
      const offset = rowOffsets[sectionIndex] ?? 0;
      if (row < offset) continue;
      const index = (row - offset) * size + column;
      const target = sections[sectionIndex];
      return target !== undefined && index < target.items.length
        ? { index, section: sectionIndex }
        : undefined;
    }
    return undefined;
  };
  return Object.freeze({ cellAt, count: rows * size, indexOf, rowOffsets, rows });
}
