<script setup lang="ts">
import { computed, onMounted, onUnmounted, useTemplateRef } from "vue";

import { commandPaletteContext } from "./command-palette-context.ts";
import type { CommandPaletteInputExpose } from "./command-palette-types.ts";

const {
  placeholder = undefined,
  ariaLabel = "Search commands",
  autofocus = false,
  disabled = false,
} = defineProps<{
  /**
   * Placeholder text shown while the search is empty.
   *
   * @default undefined
   */
  readonly placeholder?: string;

  /**
   * Accessible name of the search combobox.
   *
   * @default "Search commands"
   */
  readonly ariaLabel?: string;

  /**
   * Move focus into the input after mount.
   *
   * @default false
   */
  readonly autofocus?: boolean;

  /**
   * Disable the search input.
   *
   * @default false
   */
  readonly disabled?: boolean;
}>();

const emit = defineEmits<{
  /** Fired before palette key handling. Call `preventDefault()` to skip it. */
  keydown: [nativeEvent: KeyboardEvent];
}>();

const context = commandPaletteContext.use();
const element = useTemplateRef<HTMLInputElement>("element");
const containerProps = computed(() => context.navigation.getContainerProps());
const activeDescendant = computed(() =>
  context.open.value ? (context.activeItemId.value ?? undefined) : undefined,
);

onMounted(() => {
  context.inputElement.value = element.value;
  if (autofocus) element.value?.focus();
});

onUnmounted(() => {
  if (context.inputElement.value === element.value) context.inputElement.value = null;
});

function onInput(event: Event): void {
  if (event.target instanceof HTMLInputElement) context.setSearch(event.target.value);
  if (!context.open.value) context.setOpen(true);
}

function onFocus(event: FocusEvent): void {
  containerProps.value.onFocus(event);
}

function onKeydown(event: KeyboardEvent): void {
  emit("keydown", event);
  if (event.defaultPrevented || event.isComposing) return;
  if (event.key === "Enter") {
    if (context.open.value && context.selectActive(event)) event.preventDefault();
    return;
  }
  if (event.key === "Escape") {
    if (context.search.value !== "") {
      event.preventDefault();
      context.setSearch("");
    }
    return;
  }
  if ((event.key === "ArrowDown" || event.key === "ArrowUp") && !context.open.value) {
    event.preventDefault();
    context.setOpen(true);
    return;
  }
  if (context.open.value) containerProps.value.onKeydown(event);
}

function focus(options?: FocusOptions): void {
  element.value?.focus(options);
}

type CommandPaletteInputSetupExpose = Omit<CommandPaletteInputExpose, "element"> & {
  readonly element: typeof element;
};

const exposed = {
  element,
  focus,
} satisfies CommandPaletteInputSetupExpose;

defineExpose(exposed);
</script>

<template>
  <input
    :id="context.inputId.value"
    ref="element"
    type="text"
    role="combobox"
    autocomplete="off"
    autocorrect="off"
    spellcheck="false"
    aria-autocomplete="list"
    aria-haspopup="listbox"
    :aria-label="ariaLabel"
    :aria-expanded="context.open.value ? 'true' : 'false'"
    :aria-controls="context.listId.value"
    :aria-activedescendant="activeDescendant"
    :placeholder
    :disabled
    :value="context.search.value"
    data-vize-ui="command-palette-input"
    part="input"
    :data-state="context.state.value"
    @input="onInput"
    @focus="onFocus"
    @keydown="onKeydown"
  />
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
