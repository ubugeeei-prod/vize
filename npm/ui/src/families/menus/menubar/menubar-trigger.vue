<script setup lang="ts">
import { computed, onMounted, onUnmounted, useTemplateRef } from "vue";

import { menuLevelContext, menuTreeContext } from "../menu/menu-context.ts";
import { backwardKey, forwardKey } from "../menu/menu-dom.ts";
import { menubarContext, menubarMenuContext } from "./menubar-context.ts";
import type { MenubarTriggerExpose, MenubarTriggerSlotState } from "./menubar-types.ts";

const { disabled = false, textValue = undefined } = defineProps<{
  /**
   * Keep the trigger focusable but prevent it from opening its menu.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Typeahead text when the rendered text is not representative.
   *
   * @default undefined
   */
  readonly textValue?: string;
}>();

defineSlots<{
  /** Trigger contents. Receives menu state, value, and roving highlight. */
  default(props: MenubarTriggerSlotState): unknown;
}>();

const menubar = menubarContext.use();
const menu = menubarMenuContext.use();
const tree = menuTreeContext.use();
const level = menuLevelContext.use();
const element = useTemplateRef<HTMLButtonElement>("element");
level.hasTrigger.value = true;
const disabledState = computed(() => disabled || level.disabled.value);
const key = menu.value.value;

menubar.registry.register({
  key,
  value: { level },
  element,
  textValue: () => textValue,
  disabled: disabledState,
});

const tabStop = computed(
  () => (menubar.registry.activeKey.value ?? menubar.registry.getNavigationKey("first")) === key,
);
const highlighted = computed(() => menubar.registry.activeKey.value === key);

onMounted(() => {
  level.triggerElement.value = element.value;
});
onUnmounted(() => {
  if (level.triggerElement.value === element.value) level.triggerElement.value = null;
});

function focus(): void {
  menubar.registry.setActiveKey(key);
  element.value?.focus({ preventScroll: true });
}

function onFocus(): void {
  menubar.registry.setActiveKey(key);
}

function onPointerdown(event: PointerEvent): void {
  if (disabledState.value || event.button !== 0 || event.ctrlKey) return;
  if (event.pointerType === "touch") return;
  const opening = !level.open.value;
  focus();
  level.setOpen(opening, event, "content");
  if (opening) event.preventDefault();
}

let lastPressWasTouch = false;

function onClick(event: MouseEvent): void {
  if (disabledState.value) return;
  // Mouse presses toggle on pointer-down; taps and keyboard/AT clicks toggle here.
  if (event.detail > 0 && !lastPressWasTouch) return;
  lastPressWasTouch = false;
  level.setOpen(!level.open.value, event, event.detail === 0 ? "first" : "content");
}

function onPointerup(event: PointerEvent): void {
  lastPressWasTouch = event.pointerType === "touch";
}

function onPointerenter(event: PointerEvent): void {
  const open = menubar.value.value;
  if (disabledState.value || event.pointerType === "touch") return;
  if (open === null || open === key) return;
  focus();
  menubar.switchTo(key, event, "content");
}

function onKeydown(event: KeyboardEvent): void {
  if (event.defaultPrevented || event.isComposing) return;
  const dir = tree.dir.value;
  let handled = true;
  if (event.key === forwardKey(dir)) menubar.move(key, "next", event);
  else if (event.key === backwardKey(dir)) menubar.move(key, "previous", event);
  else if (event.key === "Home") menubar.move(key, "first", event);
  else if (event.key === "End") menubar.move(key, "last", event);
  else if (event.key === "ArrowDown" || event.key === "Enter" || event.key === " ") {
    if (!disabledState.value) level.setOpen(true, event, "first");
  } else if (event.key === "ArrowUp") {
    if (!disabledState.value) level.setOpen(true, event, "last");
  } else handled = false;
  if (handled) {
    event.preventDefault();
    return;
  }
  menubar.typeahead.typeaheadProps.onKeydown(event);
}

defineExpose({ element, focus } satisfies Omit<MenubarTriggerExpose, "element"> & {
  readonly element: typeof element;
});
</script>

<template>
  <button
    :id="level.triggerId.value"
    ref="element"
    type="button"
    role="menuitem"
    :tabindex="tabStop ? 0 : -1"
    aria-haspopup="menu"
    :aria-expanded="level.open.value ? 'true' : 'false'"
    :aria-controls="level.open.value ? level.contentId.value : undefined"
    :aria-disabled="disabledState ? 'true' : undefined"
    data-vize-ui="menubar-trigger"
    part="trigger"
    :data-state="level.state.value"
    :data-value="key"
    :data-disabled="disabledState ? 'true' : undefined"
    :data-highlighted="highlighted ? 'true' : undefined"
    @click="onClick"
    @focus="onFocus"
    @keydown="onKeydown"
    @pointerdown="onPointerdown"
    @pointerenter="onPointerenter"
    @pointerup="onPointerup"
  >
    <slot
      :dir="tree.dir.value"
      :disabled="disabledState"
      :highlighted="highlighted"
      :modal="tree.modal.value"
      :open="level.open.value"
      :state="level.state.value"
      :value="key"
    />
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
