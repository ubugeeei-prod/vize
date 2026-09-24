<script setup lang="ts">
import { computed, shallowRef } from "vue";

import { actionSheetMenuContext } from "./action-sheet-context.ts";
import type { ActionSheetItemRegistration } from "./action-sheet-context.ts";
import { dialogContext } from "../dialog/dialog-context.ts";

const { loop = true, ariaLabel = undefined } = defineProps<{
  /**
   * Wrap arrow-key focus from the last action to the first and back.
   *
   * @default true
   */
  readonly loop?: boolean;

  /**
   * Accessible name when the sheet title should not label the menu.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;
}>();

defineSlots<{
  /** ActionSheetItem actions (and optional ActionSheetCancel). */
  default(props: Record<string, never>): unknown;
}>();

const dialog = dialogContext.use();
const items = shallowRef<readonly ActionSheetItemRegistration[]>([]);
const requested = shallowRef<string | undefined>(undefined);
const enabled = () => items.value.filter((item) => !item.disabled());
// Until the user moves focus, the first enabled action is the single tab stop.
const activeId = computed(() => {
  const current = enabled().find((item) => item.id === requested.value);
  return (current ?? enabled()[0])?.id;
});

function focusItem(item: ActionSheetItemRegistration | undefined): void {
  if (item === undefined) return;
  requested.value = item.id;
  item.element()?.focus();
}

function move(offset: 1 | -1): void {
  const list = enabled();
  if (list.length === 0) return;
  const index = list.findIndex((item) => item.id === activeId.value);
  let next = index + offset;
  if (next < 0) next = loop ? list.length - 1 : 0;
  if (next >= list.length) next = loop ? 0 : list.length - 1;
  focusItem(list[next]);
}

function typeahead(character: string): void {
  const list = enabled();
  const start = list.findIndex((item) => item.id === activeId.value);
  const needle = character.toLocaleLowerCase();
  for (let step = 1; step <= list.length; step += 1) {
    const candidate = list[(start + step) % list.length];
    if (candidate?.textValue().trim().toLocaleLowerCase().startsWith(needle)) {
      focusItem(candidate);
      return;
    }
  }
}

function onKeydown(event: KeyboardEvent): void {
  if (event.altKey || event.ctrlKey || event.metaKey) return;
  const key = event.key;
  if (key === "ArrowDown") move(1);
  else if (key === "ArrowUp") move(-1);
  else if (key === "Home") focusItem(enabled()[0]);
  else if (key === "End") focusItem(enabled().at(-1));
  else if (key.length === 1 && key !== " ") typeahead(key);
  else return;
  event.preventDefault();
}

// Bound as one object: the menu is a composite widget whose key handling lives on its container.
const menuProps = computed(() => ({
  role: "menu",
  "aria-orientation": "vertical" as const,
  "aria-label": ariaLabel,
  "aria-labelledby": ariaLabel === undefined ? dialog.titleId.value : undefined,
  onKeydown,
}));

actionSheetMenuContext.provide({
  activeId,
  items,
  register: (item: ActionSheetItemRegistration) => {
    items.value = items.value.concat(item);
    return () => {
      items.value = items.value.filter((candidate) => candidate !== item);
    };
  },
  setActive: (id: string) => {
    requested.value = id;
  },
  close: (event: Event) => {
    dialog.close(event);
  },
});
</script>

<template>
  <div v-bind="menuProps" part="menu" data-vize-ui="action-sheet-menu">
    <slot />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
