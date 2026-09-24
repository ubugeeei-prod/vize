/** Compile-only assertions proving EmojiPicker inference over consumer data. */

import {
  EmojiPickerCategory,
  EmojiPickerItem,
  EmojiPickerRoot,
  emojiGlyph,
  type EmojiPickerCellSlotState,
  type EmojiPickerItemSlotState,
  type EmojiPickerSection,
  type EmojiPickerSlotState,
  type EmojiSkinTone,
} from "./emoji-picker.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

interface Glyph {
  readonly native: string;
  readonly shortcode: string;
  readonly group: "animals" | "smileys";
  readonly tones?: readonly string[];
}

declare const glyphs: readonly Glyph[];

type _Tone = Expect<Equal<EmojiSkinTone, 0 | 1 | 2 | 3 | 4 | 5>>;

EmojiPickerRoot({
  items: glyphs,
  getEmoji: (item) => {
    type _Item = Expect<Equal<typeof item, Glyph>>;
    return item.native;
  },
  getName: (item) => item.shortcode,
  getCategory: (item) => item.group,
  getSkins: (item) => item.tones,
  filter: (item, query, accessors) => {
    type _FilterItem = Expect<Equal<typeof item, Glyph>>;
    return accessors.getName(item).includes(query);
  },
  onSelect: (item, glyph) => {
    type _Selected = Expect<Equal<typeof item, Glyph>>;
    type _Glyph = Expect<Equal<typeof glyph, string>>;
  },
  "onUpdate:skinTone": (tone) => {
    type _EmittedTone = Expect<Equal<typeof tone, EmojiSkinTone>>;
  },
});

// @ts-expect-error getEmoji and getName are required accessors.
EmojiPickerRoot({ items: glyphs });

EmojiPickerRoot({
  items: glyphs,
  // @ts-expect-error accessors receive the item type.
  getEmoji: (item: string) => item,
  getName: (item: Glyph) => item.shortcode,
});

EmojiPickerRoot({
  items: glyphs,
  getEmoji: (item) => item.native,
  getName: (item) => item.shortcode,
  // @ts-expect-error skin tones are 0–5.
  skinTone: 6,
});

type RootContext = NonNullable<ReturnType<typeof EmojiPickerRoot<Glyph>>["__ctx"]>;
type _RootSlots = Expect<
  Equal<Parameters<NonNullable<RootContext["slots"]["default"]>>[0], EmojiPickerSlotState<Glyph>>
>;
type _Sections = Expect<
  Equal<EmojiPickerSlotState<Glyph>["sections"], readonly EmojiPickerSection<Glyph>[]>
>;
type CategoryContext = NonNullable<ReturnType<typeof EmojiPickerCategory<Glyph>>["__ctx"]>;
type _CellSlots = Expect<
  Equal<
    Parameters<NonNullable<CategoryContext["slots"]["default"]>>[0],
    EmojiPickerCellSlotState<Glyph>
  >
>;
type ItemContext = NonNullable<ReturnType<typeof EmojiPickerItem<Glyph>>["__ctx"]>;
type _ItemSlots = Expect<
  Equal<
    Parameters<NonNullable<ItemContext["slots"]["default"]>>[0],
    EmojiPickerItemSlotState<Glyph>
  >
>;

declare const glyph: Glyph;
const rendered: string = emojiGlyph(glyph, 3, {
  getEmoji: (item) => item.native,
  getName: (item) => item.shortcode,
});
void rendered;
