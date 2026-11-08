<script setup lang="ts">
import { nextTick, useTemplateRef, watch } from "vue";
import type { DumpLine, PageKind } from "./dumpLines";
import { withTokenOffsets } from "../../utils/withTokenOffsets";

const { lines, kind, selected, linked, marks } = defineProps<{
  lines: DumpLine[];
  kind: PageKind;
  selected: number | null;
  /** Lines covering the source cursor (reverse provenance). */
  linked: number[];
  /** Per-line gutter marks (the L3 partition kind of an op line). */
  marks?: Map<number, string>;
}>();

const emit = defineEmits<{
  hover: [number | null];
  select: [number | null];
}>();

const root = useTemplateRef<HTMLDivElement>("root");

watch(
  () => linked[0],
  async (index) => {
    if (index === undefined) return;
    await nextTick();
    root.value?.querySelector(`[data-line="${index}"]`)?.scrollIntoView({ block: "nearest" });
  },
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
        @mouseenter="() => emit('hover', line.index)"
        @focus="() => emit('hover', line.index)"
        @mouseleave="() => emit('hover', null)"
        @blur="() => emit('hover', null)"
        @click="() => toggle(line.index)"
        @keydown.enter.prevent="() => toggle(line.index)"
        @keydown.space.prevent="() => toggle(line.index)"
      >
        <span
          :class="['davinci-mark', marks?.get(line.index)]"
          :title="marks?.get(line.index)"
          aria-hidden="true"
        ></span>
        <span class="davinci-ln">{{ line.index + 1 }}</span>
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
        <span class="davinci-ln">{{ line.index + 1 }}</span>
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

<style>
/* Stage page listing: projector-legible mono text, a quiet gutter, and one
   ochre wash for the line that is linked to the source. */

.davinci-folio {
  flex: 1;
  min-height: 0;
  overflow: auto;
  padding: 12px 0;
  font-family: var(--font-mono);
  font-size: 14px;
  line-height: 1.7;
  color: var(--code-foreground);
  background: var(--code-bg);
  border: 1px solid var(--code-border);
  border-radius: 8px;
  tab-size: 2;
}

.davinci-line {
  display: flex;
  min-width: max-content;
  padding-right: 20px;
  border-left: 3px solid transparent;
}

.davinci-line.linkable {
  cursor: pointer;
}

.davinci-line.linkable:hover {
  background: var(--bg-tertiary);
}

.davinci-line.linked {
  border-left-color: var(--color-notice);
  background: var(--color-notice-bg);
}

.davinci-line.selected,
.davinci-line.selected:hover {
  border-left-color: var(--color-notice);
  background: var(--color-notice-bg);
  box-shadow: inset 0 0 0 1px var(--color-notice-border);
}

.davinci-line.davinci-section {
  margin-top: 4px;
}

/* L3 partition overlay: teal static, ochre dynamic. */
.davinci-mark {
  flex-shrink: 0;
  align-self: center;
  width: 8px;
  height: 8px;
  margin-left: 10px;
  border-radius: 50%;
}

.davinci-mark.static {
  background: var(--color-teal);
}

.davinci-mark.dynamic {
  background: var(--color-notice);
}

.davinci-ln {
  flex-shrink: 0;
  width: 3.5em;
  padding-right: 1.25em;
  text-align: right;
  color: var(--code-line-number);
  opacity: 0.7;
  user-select: none;
}

.davinci-code {
  white-space: pre;
}

/* Tokens */
.tk-section {
  color: var(--hl-type);
  font-weight: 600;
}

.tk-key {
  color: var(--hl-property);
}

.tk-mnemonic {
  color: var(--hl-keyword);
  font-weight: 600;
}

.tk-tag {
  color: var(--hl-tag);
}

.tk-string {
  color: var(--hl-string);
}

.tk-span {
  color: var(--code-line-number);
}

.tk-number {
  color: var(--hl-number);
}

.tk-call {
  color: var(--hl-function);
}

.tk-absent,
.tk-punct {
  color: var(--hl-delimiter);
}

.tk-attr {
  color: var(--hl-property);
}

.tk-directive {
  color: var(--hl-directive);
}

.tk-mustache {
  color: var(--hl-function);
}

.tk-static,
.tk-dynamic {
  padding: 0 6px;
  border-radius: 4px;
  font-weight: 600;
}

.tk-static {
  color: var(--color-teal);
  background: var(--color-teal-bg);
}

.tk-dynamic {
  color: var(--color-notice);
  background: var(--color-notice-bg);
}

/* Page difference against the previous page of the same stage */
.davinci-diff,
.davinci-remarks {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.davinci-diff-summary {
  margin: 0;
  font-size: 14px;
  color: var(--text-secondary);
}

.davinci-diff-mark {
  flex-shrink: 0;
  width: 1.5em;
  color: var(--text-muted);
  user-select: none;
}

.davinci-line.diff-add {
  background: var(--color-success-bg);
}

.davinci-line.diff-add .davinci-diff-mark {
  color: var(--color-success);
}

.davinci-line.diff-remove {
  background: var(--color-error-bg);
}

.davinci-line.diff-remove .davinci-diff-mark {
  color: var(--color-error);
}

/* Optimization remarks */
.davinci-remarks-empty {
  max-width: 64ch;
  padding: 20px 22px;
  border: 1px dashed var(--border-primary);
  border-radius: 8px;
}

.davinci-remarks-title {
  margin: 0 0 6px;
  font-size: 15px;
  font-weight: 600;
}

.davinci-remark-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin: 0;
  padding: 0;
  overflow: auto;
  list-style: none;
}

.davinci-remark {
  display: grid;
  grid-template-columns: auto auto 1fr auto;
  align-items: baseline;
  gap: 10px;
  padding: 8px 12px;
  font-size: 14px;
  border: 1px solid var(--border-primary);
  border-radius: 8px;
}

.davinci-remark-outcome {
  padding: 1px 6px;
  font-family: var(--font-mono);
  font-size: 12px;
  border-radius: 4px;
}

.davinci-remark.applied .davinci-remark-outcome {
  color: var(--color-teal);
  background: var(--color-teal-bg);
}

.davinci-remark.missed .davinci-remark-outcome {
  color: var(--color-notice);
  background: var(--color-notice-bg);
}

.davinci-remark.analysis .davinci-remark-outcome {
  color: var(--text-secondary);
  background: var(--bg-tertiary);
}

.davinci-remark-pass {
  font-family: var(--font-mono);
  color: var(--text-secondary);
}

.davinci-remark-message {
  display: flex;
  flex-wrap: wrap;
  gap: 4px 8px;
  align-items: baseline;
}

.davinci-remark-arg {
  font-family: var(--font-mono);
  font-size: 12px;
  color: var(--text-muted);
}
</style>
