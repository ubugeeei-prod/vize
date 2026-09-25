<script setup lang="ts">
import { useTemplateRef } from "vue";

import { commandPaletteContext } from "./command-palette-context.ts";
import type { CommandPaletteListExpose, CommandPaletteSlotState } from "./command-palette-types.ts";

const { ariaLabel = "Commands" } = defineProps<{
  /**
   * Accessible name of the listbox.
   *
   * @default "Commands"
   */
  readonly ariaLabel?: string;
}>();

defineSlots<{
  /** Items, groups, empty, and loading parts. */
  default?(props: Omit<CommandPaletteSlotState, "recentCommands">): unknown;
}>();

const context = commandPaletteContext.use();
const element = useTemplateRef<HTMLDivElement>("element");

type CommandPaletteListSetupExpose = Omit<CommandPaletteListExpose, "element"> & {
  readonly element: typeof element;
};

const exposed = {
  element,
} satisfies CommandPaletteListSetupExpose;

defineExpose(exposed);
// Options are reached through aria-activedescendant on the input, so they are
// intentionally not focusable; the role is bound as data to reflect that.
const listboxRole = { role: "listbox" } as const;
</script>

<template>
  <div
    :id="context.listId.value"
    ref="element"
    v-bind="listboxRole"
    :aria-label="ariaLabel"
    :aria-busy="context.loading.value ? 'true' : undefined"
    :hidden="context.open.value ? undefined : true"
    data-vize-ui="command-palette-list"
    part="list"
    :data-state="context.state.value"
    :data-empty="context.resultCount.value === 0 ? 'true' : undefined"
  >
    <slot
      :commands="context.commands.value"
      :loading="context.loading.value"
      :open="context.open.value"
      :result-count="context.resultCount.value"
      :search="context.search.value"
    />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
