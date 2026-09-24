<script setup lang="ts">
import { computed, shallowRef } from "vue";

import { useDeterministicId } from "../../foundations/id/deterministic-id.ts";
import { commandPaletteContext, commandPaletteGroupContext } from "./command-palette-context.ts";
import type { CommandPaletteGroupSlotState } from "./command-palette-types.ts";

const { heading, forceMount = false } = defineProps<{
  /**
   * Visible group heading, also used as the group's accessible name.
   *
   * @default required
   */
  readonly heading: string;

  /**
   * Keep the group visible even when no item matches the search.
   *
   * @default false
   */
  readonly forceMount?: boolean;
}>();

defineSlots<{
  /** Group items. Receives the heading and visible item count. */
  default?(props: CommandPaletteGroupSlotState): unknown;

  /** Custom heading contents. Receives the heading and visible item count. */
  heading?(props: CommandPaletteGroupSlotState): unknown;
}>();

const context = commandPaletteContext.use();
const headingId = useDeterministicId({ hint: "command-palette-group" });
const members = shallowRef<ReadonlySet<string>>(new Set());
const resultCount = computed(() => {
  let count = 0;
  for (const member of members.value) if (context.isVisible(member)) count += 1;
  return count;
});
const visible = computed(() => forceMount || resultCount.value > 0);
const slotState = computed<CommandPaletteGroupSlotState>(() => ({
  heading,
  resultCount: resultCount.value,
}));

function addMember(id: string): () => void {
  const next = new Set(members.value);
  next.add(id);
  members.value = next;
  return () => {
    const next = new Set(members.value);
    next.delete(id);
    members.value = next;
  };
}

commandPaletteGroupContext.provide({ addMember, headingId });
</script>

<template>
  <div
    role="group"
    :aria-labelledby="headingId"
    :hidden="visible ? undefined : true"
    data-vize-ui="command-palette-group"
    part="group"
    :data-empty="resultCount === 0 ? 'true' : undefined"
  >
    <div
      :id="headingId"
      aria-hidden="true"
      data-vize-ui="command-palette-group-heading"
      part="group-heading"
    >
      <slot name="heading" v-bind="slotState">{{ heading }}</slot>
    </div>
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
