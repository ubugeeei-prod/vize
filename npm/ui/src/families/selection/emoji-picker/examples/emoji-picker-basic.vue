<!-- Emoji picker over consumer-supplied emoji data with search, categories, skin tones, and a preview. -->
<script setup lang="ts">
import { ref } from "vue";

import {
  EmojiPicker,
  EmojiPickerCategory,
  EmojiPickerEmpty,
  EmojiPickerGrid,
  EmojiPickerItem,
  EmojiPickerPreview,
  EmojiPickerSearch,
  EmojiPickerSkinTone,
} from "../emoji-picker.ts";

interface Emoji {
  readonly emoji: string;
  readonly name: string;
  readonly category: "smileys" | "animals";
  readonly keywords: readonly string[];
  readonly skins?: readonly string[];
}

const emoji: readonly Emoji[] = [
  { emoji: "😀", name: "grinning face", category: "smileys", keywords: ["happy"] },
  { emoji: "😂", name: "tears of joy", category: "smileys", keywords: ["laugh"] },
  {
    emoji: "👋",
    name: "waving hand",
    category: "smileys",
    keywords: ["hello", "bye"],
    skins: ["👋", "👋🏻", "👋🏼", "👋🏽", "👋🏾", "👋🏿"],
  },
  { emoji: "🐱", name: "cat face", category: "animals", keywords: ["pet"] },
  { emoji: "🐶", name: "dog face", category: "animals", keywords: ["puppy", "pet"] },
];
const categories = [
  { id: "smileys", label: "Smileys & people" },
  { id: "animals", label: "Animals" },
];
const picked = ref("");
</script>

<template>
  <div>
    <EmojiPicker
      v-slot="{ sections }"
      :items="emoji"
      :categories
      :columns="4"
      :get-emoji="(item: Emoji) => item.emoji"
      :get-name="(item: Emoji) => item.name"
      :get-category="(item: Emoji) => item.category"
      :get-keywords="(item: Emoji) => item.keywords"
      :get-skins="(item: Emoji) => item.skins"
      @select="(_item: Emoji, glyph: string) => (picked = glyph)"
    >
      <EmojiPickerSearch placeholder="Search emoji" />
      <EmojiPickerSkinTone />
      <EmojiPickerGrid>
        <EmojiPickerCategory
          v-for="section in sections"
          :key="section.id"
          v-slot="{ item, index }"
          :section
        >
          <EmojiPickerItem :item :index />
        </EmojiPickerCategory>
      </EmojiPickerGrid>
      <EmojiPickerEmpty>No emoji found</EmojiPickerEmpty>
      <EmojiPickerPreview />
    </EmojiPicker>
    <output>Picked: {{ picked || "none" }}</output>
  </div>
</template>
