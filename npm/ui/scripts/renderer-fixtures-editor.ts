/** Editor fixtures compiled by every supported renderer lane. */
export const editorRendererFixtures = [
  {
    filename: "RichTextConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import {
  createDefaultRichTextSchema,
  createRichTextCommands,
  emptyRichTextDoc,
  isMarkActive,
  RichTextBubbleMenu,
  RichTextContent,
  RichTextRoot,
  RichTextToolbar,
  RichTextToolbarButton,
} from "./families/editor/rich-text/rich-text.ts";

const schema = createDefaultRichTextSchema();
const commands = createRichTextCommands(schema);
const doc = ref(emptyRichTextDoc(schema));
</script>

<template>
  <RichTextRoot v-model="doc" :schema>
    <RichTextToolbar aria-label="Formatting">
      <RichTextToolbarButton
        :command="commands.toggleMark('bold')"
        :active="(state) => isMarkActive(state, 'bold')"
        aria-label="Bold"
      >
        B
      </RichTextToolbarButton>
      <RichTextToolbarButton :command="commands.undo" aria-label="Undo">Undo</RichTextToolbarButton>
    </RichTextToolbar>
    <RichTextContent aria-label="Body" placeholder="Write…" />
    <RichTextBubbleMenu>
      <RichTextToolbarButton :command="commands.toggleMark('italic')" aria-label="Italic">I</RichTextToolbarButton>
    </RichTextBubbleMenu>
  </RichTextRoot>
</template>
`,
  },
] as const;
