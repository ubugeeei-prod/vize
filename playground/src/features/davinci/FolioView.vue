<script setup lang="ts">
import { nextTick, useTemplateRef, watch } from "vue";
import type { FolioLine, PageKind } from "./folioLines";

const props = defineProps<{
  lines: FolioLine[];
  kind: PageKind;
  selected: number | null;
  /** Lines covering the source cursor (reverse provenance). */
  linked: number[];
}>();

const emit = defineEmits<{
  hover: [number | null];
  select: [number | null];
}>();

const root = useTemplateRef<HTMLDivElement>("root");

watch(
  () => props.linked[0],
  async (index) => {
    if (index === undefined) return;
    await nextTick();
    root.value?.querySelector(`[data-line="${index}"]`)?.scrollIntoView({ block: "nearest" });
  },
);

function spanTitle(line: FolioLine): string | undefined {
  return line.span ? `Authored bytes ${line.span.start}–${line.span.end}` : undefined;
}

function toggle(index: number) {
  emit("select", props.selected === index ? null : index);
}
</script>

<template>
  <div ref="root" :class="['davinci-folio', `page-${kind}`]" aria-label="Stage page">
    <template v-for="line in lines" :key="line.index">
      <div
        v-if="line.span"
        :class="[
          'davinci-line',
          'linkable',
          { selected: selected === line.index, linked: linked.includes(line.index) },
        ]"
        role="button"
        tabindex="0"
        :data-line="line.index"
        :aria-pressed="selected === line.index"
        :title="spanTitle(line)"
        @mouseenter="emit('hover', line.index)"
        @focus="emit('hover', line.index)"
        @mouseleave="emit('hover', null)"
        @blur="emit('hover', null)"
        @click="toggle(line.index)"
        @keydown.enter.prevent="toggle(line.index)"
        @keydown.space.prevent="toggle(line.index)"
      >
        <span class="davinci-ln">{{ line.index + 1 }}</span>
        <span class="davinci-code"
          ><span v-for="(token, ti) in line.tokens" :key="ti" :class="`tk-${token.type}`">{{
            token.text
          }}</span></span
        >
      </div>
      <div
        v-else
        :class="['davinci-line', { 'davinci-section': line.tokens[0]?.type === 'section' }]"
        :data-line="line.index"
      >
        <span class="davinci-ln">{{ line.index + 1 }}</span>
        <span class="davinci-code"
          ><span v-for="(token, ti) in line.tokens" :key="ti" :class="`tk-${token.type}`">{{
            token.text
          }}</span
          ><template v-if="line.tokens.length === 0">&#160;</template></span
        >
      </div>
    </template>
  </div>
</template>
