<script setup lang="ts">
import { computed } from "vue";

import { renderMarkdown } from "./markdown.js";
import type { MarkdownProps } from "./markdown-types.js";

const props = defineProps<MarkdownProps>();

defineOptions({ name: "Markdown" });

const lines = computed(() =>
  renderMarkdown(
    props.source,
    {
      headingFg: props.headingFg,
      linkFg: props.linkFg,
      quoteFg: props.quoteFg,
      codeFg: props.codeFg,
      providers: props.providers,
    },
    props.renderer,
  ),
);
const containerStyle = { flexDirection: "column" } as const;
const lineStyle = { flexDirection: "row" } as const;
</script>

<template>
  <div :style="containerStyle">
    <div v-for="line in lines" :key="line.key" :style="lineStyle">
      <text
        v-for="(span, index) in line.spans"
        :key="`${line.key}-${index}`"
        :text="span.text"
        :fg="span.fg"
        :bold="span.bold"
        :italic="span.italic"
        :underline="span.underline"
        :dim="span.dim"
      />
    </div>
  </div>
</template>
