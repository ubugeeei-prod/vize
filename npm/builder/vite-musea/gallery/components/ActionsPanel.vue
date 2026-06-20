<script setup lang="ts">
import { computed, ref } from "vue";
import hljs from "highlight.js/lib/core";
import json from "highlight.js/lib/languages/json";
import { mdiChevronUp, mdiChevronDown } from "@mdi/js";
import { useActions, type ActionEvent } from "../composables/useActions";
import MdiIcon from "./MdiIcon.vue";

const props = withDefaults(
  defineProps<{
    captureEvents?: string[];
  }>(),
  {
    captureEvents: () => [],
  },
);

hljs.registerLanguage("json", json);

const { events, clear } = useActions();
const expandedIndex = ref<number | null>(null);

const reversedEvents = computed(() => [...events.value].reverse());
const tracksMousemove = computed(() =>
  props.captureEvents.some((eventName) => eventName === "mousemove"),
);
const captureLabel = computed(() =>
  tracksMousemove.value ? "mousemove enabled" : "standard capture",
);
const totalLabel = computed(() => `${events.value.length} total`);

function formatTarget(target?: string): string {
  return target ? target.toLowerCase() : "document";
}

function toggleExpand(index: number) {
  expandedIndex.value = expandedIndex.value === index ? null : index;
}

function formatTime(timestamp: number): string {
  const d = new Date(timestamp);
  return d.toLocaleTimeString("en", {
    hour12: false,
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
    fractionalSecondDigits: 3,
  });
}

function formatRawEvent(event: ActionEvent): string {
  if (!event.rawEvent) {
    return JSON.stringify({ target: event.target, value: event.value }, null, 2);
  }
  return JSON.stringify(event.rawEvent, null, 2);
}

function highlightJson(str: string): string {
  return hljs.highlight(str, { language: "json" }).value;
}
</script>

<template>
  <div class="actions-panel">
    <div class="actions-header">
      <div class="actions-header-copy">
        <span class="actions-header-title">Captured Events</span>
        <span class="actions-header-meta">
          <span class="actions-count">{{ totalLabel }}</span>
          <span
            class="actions-capture-mode"
            :class="{ 'actions-capture-mode--active': tracksMousemove }"
          >
            {{ captureLabel }}
          </span>
        </span>
      </div>
      <button v-if="events.length > 0" type="button" class="actions-clear-btn" @click="clear()">
        Clear
      </button>
    </div>

    <div v-if="events.length === 0" class="actions-empty">
      <p>No events captured yet.</p>
      <p class="actions-hint">
        {{
          tracksMousemove
            ? "Move the pointer or interact with the component to see events here."
            : "Interact with the component to see events here."
        }}
      </p>
      <p v-if="!tracksMousemove" class="actions-note">
        Add <code>action-events="mousemove"</code> to the <code>&lt;art&gt;</code> metadata to opt
        in.
      </p>
    </div>

    <div v-else class="actions-list">
      <div
        v-for="(event, index) in reversedEvents"
        :key="index"
        class="action-item"
        :class="{ expanded: expandedIndex === index }"
        @click="toggleExpand(index)"
      >
        <div class="action-row">
          <span class="action-time">{{ formatTime(event.timestamp) }}</span>
          <span class="action-type" :class="event.source">{{ event.name }}</span>
          <span class="action-target">{{ formatTarget(event.target) }}</span>
          <MdiIcon
            class="action-expand-icon"
            :path="expandedIndex === index ? mdiChevronUp : mdiChevronDown"
            :size="12"
          />
        </div>
        <div v-if="expandedIndex === index" class="action-detail">
          <pre
            class="action-raw hljs"
          ><code v-html="highlightJson(formatRawEvent(event))"></code></pre>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.actions-panel {
  font-size: 0.75rem;
  height: 100%;
  display: flex;
  flex-direction: column;
}

.actions-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  padding: 0.75rem 1rem 0.625rem;
  border-bottom: 1px solid var(--musea-border-subtle);
  background: transparent;
}

.actions-header-copy {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 0.1875rem;
  min-width: 0;
}

.actions-header-title {
  color: var(--musea-text);
  font-size: 0.75rem;
  font-weight: 600;
}

.actions-header-meta {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex-wrap: wrap;
  min-width: 0;
}

.actions-count {
  font-size: 0.6875rem;
  color: var(--musea-text-muted);
}

.actions-capture-mode {
  padding: 0.125rem 0.4375rem;
  border-radius: 999px;
  background: var(--musea-bg-secondary);
  color: var(--musea-text-muted);
  font-size: 0.625rem;
  font-weight: 700;
  letter-spacing: 0.04em;
  text-transform: uppercase;
}

.actions-capture-mode--active {
  background: var(--musea-accent-subtle);
  color: var(--musea-accent);
}

.actions-clear-btn {
  padding: 0.125rem 0.375rem;
  border: 1px solid var(--musea-border);
  border-radius: var(--musea-radius-sm);
  background: transparent;
  color: var(--musea-text-muted);
  font-size: 0.625rem;
  cursor: pointer;
  transition: all var(--musea-transition);
}

.actions-clear-btn:hover {
  border-color: var(--musea-text-muted);
  color: var(--musea-text);
}

.actions-empty {
  padding: 1.25rem 1rem 1.5rem;
  text-align: center;
  color: var(--musea-text-muted);
  font-size: 0.75rem;
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
  margin: auto 0;
}

.actions-hint {
  font-size: 0.6875rem;
  opacity: 0.7;
}

.actions-note {
  font-size: 0.6875rem;
  color: var(--musea-text-muted);
}

.actions-note code {
  font-family: var(--musea-font-mono, monospace);
  font-size: 0.625rem;
  padding: 0.125rem 0.25rem;
  border-radius: var(--musea-radius-sm);
  background: var(--musea-bg-secondary);
  border: 1px solid var(--musea-border-subtle);
}

.actions-list {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 0.5rem;
}

.action-item {
  border: 1px solid transparent;
  border-radius: var(--musea-radius-md);
  cursor: pointer;
  transition:
    background var(--musea-transition),
    border-color var(--musea-transition);
}

.action-item + .action-item {
  margin-top: 0.375rem;
}

.action-item:hover {
  background: color-mix(in srgb, var(--musea-bg-secondary) 72%, transparent);
  border-color: var(--musea-border-subtle);
}

.action-item.expanded {
  background: color-mix(in srgb, var(--musea-bg-secondary) 82%, transparent);
  border-color: var(--musea-border-subtle);
}

.action-row {
  display: grid;
  grid-template-columns: auto auto minmax(0, 1fr) auto;
  align-items: center;
  column-gap: 0.625rem;
  padding: 0.5rem 0.75rem;
  font-size: 0.6875rem;
}

.action-time {
  color: var(--musea-text-muted);
  font-family: var(--musea-font-mono, monospace);
  font-size: 0.625rem;
  flex-shrink: 0;
}

.action-type {
  padding: 0.0625rem 0.25rem;
  border-radius: 2px;
  font-size: 0.5625rem;
  font-weight: 600;
  flex-shrink: 0;
  background: rgba(59, 130, 246, 0.15);
  color: #60a5fa;
}

.action-type.vue {
  background: rgba(52, 211, 153, 0.15);
  color: #34d399;
}

.action-target {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--musea-text-secondary);
  font-family: var(--musea-font-mono, monospace);
  font-size: 0.625rem;
}

.action-expand-icon {
  width: 12px;
  height: 12px;
  margin-left: auto;
  color: var(--musea-text-muted);
  flex-shrink: 0;
}

.action-detail {
  padding: 0 0.75rem 0.75rem 0.75rem;
}

.action-raw {
  background: var(--musea-bg-tertiary);
  border: 1px solid var(--musea-border);
  border-radius: var(--musea-radius-sm);
  padding: 0.375rem 0.5rem;
  font-family: var(--musea-font-mono, monospace);
  font-size: 0.5625rem;
  color: var(--musea-text-secondary);
  overflow-x: auto;
  white-space: pre;
  margin: 0;
  max-height: 150px;
  overflow-y: auto;
}
</style>
