import { h } from "vue";
import type { VNode } from "vue";

import EmojiPickerCategory from "./emoji-picker-category.vue";
import EmojiPickerEmpty from "./emoji-picker-empty.vue";
import EmojiPickerGrid from "./emoji-picker-grid.vue";
import EmojiPickerItem from "./emoji-picker-item.vue";
import EmojiPickerPreview from "./emoji-picker-preview.vue";
import EmojiPickerRoot from "./emoji-picker-root.vue";
import EmojiPickerSearch from "./emoji-picker-search.vue";
import EmojiPickerSkinTone from "./emoji-picker-skin-tone.vue";
import type { EmojiPickerSection } from "./emoji-picker-model.ts";
import type { EmojiPickerCellSlotState, EmojiPickerSlotState } from "./emoji-picker-types.ts";

/** Tiny fixture emoji record used by tests only. */
export interface FixtureEmoji {
  readonly emoji: string;
  readonly name: string;
  readonly category: string;
  readonly keywords?: readonly string[];
  readonly skins?: readonly string[];
}

// Three columns lay out as:
// smileys: [grinning, joy, wave] / [smile]
// animals: [cat, dog]
export const fixtureEmoji: readonly FixtureEmoji[] = [
  { category: "smileys", emoji: "😀", name: "grinning" },
  { category: "smileys", emoji: "😂", keywords: ["laugh"], name: "joy" },
  {
    category: "smileys",
    emoji: "👋",
    keywords: ["hello"],
    name: "wave",
    skins: ["👋", "👋🏻", "👋🏼", "👋🏽", "👋🏾", "👋🏿"],
  },
  { category: "smileys", emoji: "😊", name: "smile" },
  { category: "animals", emoji: "🐱", name: "cat" },
  { category: "animals", emoji: "🐶", keywords: ["puppy"], name: "dog" },
];

/** Render a complete picker over `fixtureEmoji`. */
export function renderEmojiPickerTree(props: Record<string, unknown> = {}): VNode {
  return h(
    EmojiPickerRoot<FixtureEmoji>,
    {
      categories: [
        { id: "smileys", label: "Smileys" },
        { id: "animals", label: "Animals" },
      ],
      columns: 3,
      getCategory: (item: FixtureEmoji) => item.category,
      getEmoji: (item: FixtureEmoji) => item.emoji,
      getKeywords: (item: FixtureEmoji) => item.keywords ?? [],
      getName: (item: FixtureEmoji) => item.name,
      getSkins: (item: FixtureEmoji) => item.skins,
      id: "emoji",
      items: fixtureEmoji,
      ...props,
    },
    {
      default: (state: EmojiPickerSlotState<FixtureEmoji>) => [
        h(EmojiPickerSearch, { placeholder: "Search" }),
        h(EmojiPickerSkinTone),
        h(EmojiPickerGrid, null, () =>
          state.sections.map((section: EmojiPickerSection<FixtureEmoji>) =>
            h(
              EmojiPickerCategory<FixtureEmoji>,
              { key: section.id, section },
              {
                default: (cell: EmojiPickerCellSlotState<FixtureEmoji>) =>
                  h(EmojiPickerItem<FixtureEmoji>, { index: cell.index, item: cell.item }),
              },
            ),
          ),
        ),
        h(EmojiPickerEmpty, null, () => "No emoji found"),
        h(EmojiPickerPreview),
      ],
    },
  );
}
