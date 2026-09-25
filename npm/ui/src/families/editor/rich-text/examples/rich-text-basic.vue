<!-- Rich-text editor with a formatting toolbar, a selection bubble menu, and v-model HTML output. -->
<script setup lang="ts">
import { computed, ref } from "vue";

import {
  RichTextBubbleMenu,
  RichTextContent,
  RichTextRoot,
  RichTextToolbar,
  RichTextToolbarButton,
  createDefaultRichTextSchema,
  createRichTextCommands,
  isBlockActive,
  isMarkActive,
  richTextDocFromText,
  richTextToHtml,
} from "../rich-text.ts";

const schema = createDefaultRichTextSchema();
const commands = createRichTextCommands(schema);
const doc = ref(
  richTextDocFromText(schema, "Release notes\nType ## for a heading or - for a list."),
);
const html = computed(() => richTextToHtml(schema, doc.value));
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
      <RichTextToolbarButton
        :command="commands.toggleBlockType('heading', { level: 2 })"
        :active="(state) => isBlockActive(state, 'heading', { level: 2 })"
        aria-label="Heading"
      >
        H2
      </RichTextToolbarButton>
      <RichTextToolbarButton :command="commands.toggleWrap('bulletList')" aria-label="Bullet list">
        List
      </RichTextToolbarButton>
      <RichTextToolbarButton :command="commands.undo" aria-label="Undo">Undo</RichTextToolbarButton>
    </RichTextToolbar>
    <RichTextContent aria-label="Release notes" placeholder="Write something…" />
    <RichTextBubbleMenu>
      <RichTextToolbarButton :command="commands.toggleMark('italic')" aria-label="Italic">
        I
      </RichTextToolbarButton>
    </RichTextBubbleMenu>
  </RichTextRoot>
  <output>{{ html }}</output>
</template>
