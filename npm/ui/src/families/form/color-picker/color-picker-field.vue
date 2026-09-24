<script setup lang="ts">
import { computed, shallowRef, useTemplateRef, watch } from "vue";
import type { ComputedRef, ShallowRef } from "vue";

import { formatColor, parseColor } from "./color-picker-color.ts";
import type { ColorFormat } from "./color-picker-color.ts";
import { colorPickerContext } from "./color-picker-context.ts";
import type { ColorPickerFieldExpose, ColorPickerFieldState } from "./color-picker-types.ts";

const {
  format = undefined,
  preserveAlpha = false,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
  placeholder = undefined,
} = defineProps<{
  /**
   * Serialization shown in the field. Defaults to the root `format`.
   *
   * @default undefined
   */
  readonly format?: ColorFormat;

  /**
   * Keep the current alpha when the typed color has no alpha component
   * (for example typing `#ff0000` into a half-transparent picker).
   *
   * @default false
   */
  readonly preserveAlpha?: boolean;

  /**
   * Accessible name when no `<label>` or `aria-labelledby` supplies one.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids that label the field.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;

  /**
   * Space-separated ids that describe the field.
   *
   * @default undefined
   */
  readonly ariaDescribedby?: string;

  /**
   * Native placeholder text.
   *
   * @default undefined
   */
  readonly placeholder?: string;
}>();

const emit = defineEmits<{
  /** Fired when a committed draft cannot be parsed and the field reverts. */
  invalid: [draft: string, nativeEvent: Event | null];
}>();

const context = colorPickerContext.use();
const element = useTemplateRef<HTMLInputElement>("element");
const formatState = computed<ColorFormat>(() => format ?? context.format.value);
const formatted = computed(() => formatColor(context.color.value, formatState.value));
const draft = shallowRef(formatted.value);
const editing = shallowRef(false);
const state = computed<ColorPickerFieldState>(() =>
  draft.value.trim().length === 0 || parseColor(draft.value) !== null ? "valid" : "invalid",
);

watch(formatted, (next) => {
  if (!editing.value) draft.value = next;
});

function hasExplicitAlpha(text: string): boolean {
  const source = text.trim();
  if (source.startsWith("#")) return source.length === 5 || source.length === 9;
  return source.includes("/") || source.split(",").length === 4;
}

function revert(): void {
  editing.value = false;
  draft.value = formatted.value;
}

function readDraft(): string {
  return draft.value;
}

function commitDraft(nativeEvent: Event | null): boolean {
  return commitText(readDraft(), nativeEvent);
}

function commitText(text: string, nativeEvent: Event | null): boolean {
  const parsed = parseColor(text);
  if (parsed === null) {
    if (text.trim() !== formatted.value) emit("invalid", text, nativeEvent);
    revert();
    return false;
  }
  editing.value = false;
  if (context.editable.value) {
    const next =
      preserveAlpha && !hasExplicitAlpha(text)
        ? { ...parsed, alpha: context.color.value.alpha }
        : parsed;
    context.setColor(next, "field", nativeEvent);
    context.commit("field", nativeEvent);
  }
  draft.value = formatted.value;
  return true;
}

function onInput(event: Event): void {
  if (!(event.currentTarget instanceof HTMLInputElement)) return;
  editing.value = true;
  draft.value = event.currentTarget.value;
}

function onKeydown(event: KeyboardEvent): void {
  if (event.key === "Enter") {
    commitDraft(event);
  } else if (event.key === "Escape" && editing.value) {
    event.preventDefault();
    revert();
  }
}

function onBlur(event: FocusEvent): void {
  if (editing.value) commitDraft(event);
}

function focus(options?: FocusOptions): void {
  element.value?.focus(options);
}

type ColorPickerFieldSetupExpose = Omit<ColorPickerFieldExpose, "draft" | "element" | "state"> & {
  readonly draft: ShallowRef<string>;
  readonly element: typeof element;
  readonly state: ComputedRef<ColorPickerFieldState>;
};

const exposed = {
  commit: () => commitDraft(null),
  draft,
  element,
  focus,
  revert,
  state,
} satisfies ColorPickerFieldSetupExpose;

defineExpose(exposed);
</script>

<template>
  <input
    :id="context.getPartId('field')"
    ref="element"
    type="text"
    autocomplete="off"
    autocapitalize="off"
    autocorrect="off"
    spellcheck="false"
    :value="draft"
    :placeholder
    :disabled="context.disabled.value"
    :readonly="context.readOnly.value"
    :aria-label="ariaLabel"
    :aria-labelledby="ariaLabelledby"
    :aria-describedby="ariaDescribedby"
    :aria-invalid="state === 'invalid' ? 'true' : undefined"
    data-vize-ui="color-picker-field"
    part="field"
    :data-state="state"
    :data-format="formatState"
    :data-disabled="context.disabled.value ? 'true' : undefined"
    @input="onInput"
    @keydown="onKeydown"
    @blur="onBlur"
  />
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
