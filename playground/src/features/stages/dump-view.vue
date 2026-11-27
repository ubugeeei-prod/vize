<script setup lang="ts">
import "./dump-view-styles.vue";
import { computed, nextTick, useTemplateRef, watch } from "vue";
import { irNotation } from "./irNotation";
import type { DumpLine, PageKind } from "./dumpLines";
import { withTokenOffsets } from "../../utils/withTokenOffsets";
import { traceMatches } from "./sourceTrace";
import type { Range } from "./offsets";

const {
  lines,
  kind,
  selected,
  linked,
  marks,
  notation = "raw",
  reveal,
} = defineProps<{
  lines: DumpLine[];
  kind: PageKind;
  selected: number | null;
  /** Lines covering the source cursor (reverse provenance). */
  linked: number[];
  /** Per-line gutter marks (the L3 partition kind of an op line). */
  marks?: Map<number, string>;
  notation?: "dsl" | "raw";
  /** Explicit selection only; hover highlights never trigger scroll. */
  reveal?: Range | null;
}>();

const displayLines = computed(() => (notation === "dsl" ? irNotation(kind, lines) : lines));

const emit = defineEmits<{
  highlight: [number | null];
  select: [number | null];
}>();

const root = useTemplateRef<HTMLDivElement>("root");

const revealLine = computed(() =>
  reveal === undefined ? linked[0] : (selected ?? traceMatches(lines, reveal, kind)[0]),
);
watch(
  () => [revealLine.value, notation, lines] as const,
  async ([index]) => {
    if (index === undefined) return;
    await nextTick();
    root.value
      ?.querySelector(`[data-line="${index}"]`)
      ?.scrollIntoView({ block: "nearest", inline: "nearest" });
  },
  { immediate: true },
);

function spanTitle(line: DumpLine): string | undefined {
  return line.span ? `Authored bytes ${line.span.start}–${line.span.end}` : undefined;
}

function toggle(index: number) {
  emit("select", selected === index ? null : index);
}
</script>

<template>
  <div ref="root" :class="['davinci-folio', `page-${kind}`]" aria-label="Stage page">
    <template v-for="(line, displayIndex) in displayLines" :key="line.index">
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
        @mouseenter="() => emit('highlight', line.index)"
        @focus="() => emit('highlight', line.index)"
        @mouseleave="() => emit('highlight', null)"
        @blur="() => emit('highlight', null)"
        @click="() => toggle(line.index)"
        @keydown.enter.prevent="() => toggle(line.index)"
        @keydown.space.prevent="() => toggle(line.index)"
      >
        <span
          :class="['davinci-mark', marks?.get(line.index)]"
          :title="marks?.get(line.index)"
          aria-hidden="true"
        ></span>
        <span class="davinci-ln">{{ displayIndex + 1 }}</span>
        <span class="davinci-code"
          ><span
            v-for="token in withTokenOffsets(line.tokens)"
            :key="token.offset"
            :class="`tk-${token.type}`"
            >{{ token.text }}</span
          ></span
        >
      </div>
      <div
        v-else
        :class="['davinci-line', { 'davinci-section': line.tokens[0]?.type === 'section' }]"
        :data-line="line.index"
      >
        <span
          :class="['davinci-mark', marks?.get(line.index)]"
          :title="marks?.get(line.index)"
          aria-hidden="true"
        ></span>
        <span class="davinci-ln">{{ displayIndex + 1 }}</span>
        <span class="davinci-code"
          ><span
            v-for="token in withTokenOffsets(line.tokens)"
            :key="token.offset"
            :class="`tk-${token.type}`"
            >{{ token.text }}</span
          ><template v-if="line.tokens.length === 0">&#160;</template></span
        >
      </div>
    </template>
  </div>
</template>
