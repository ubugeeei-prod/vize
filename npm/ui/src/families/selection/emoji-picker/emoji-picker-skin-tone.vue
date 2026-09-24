<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import { emojiPickerContext } from "./emoji-picker-context.ts";
import { toSkinTone } from "./emoji-picker-model.ts";
import type { EmojiSkinTone } from "./emoji-picker-model.ts";

const {
  ariaLabel = "Skin tone",
  labels = ["Default", "Light", "Medium-light", "Medium", "Medium-dark", "Dark"],
} = defineProps<{
  /**
   * Accessible name of the radio group.
   *
   * @default "Skin tone"
   */
  readonly ariaLabel?: string;

  /**
   * Accessible names of tones 0–5.
   *
   * @default ["Default", "Light", "Medium-light", "Medium", "Medium-dark", "Dark"]
   */
  readonly labels?: readonly string[];
}>();

defineSlots<{
  /** Content of one tone swatch. Receives the tone, its label, and checked state. */
  default?(props: {
    readonly tone: EmojiSkinTone;
    readonly label: string;
    readonly checked: boolean;
  }): unknown;
}>();

/** One rendered tone option. */
interface ToneOption {
  readonly tone: EmojiSkinTone;
  readonly label: string;
  readonly checked: boolean;
  /** Radio role and selection handler bound onto the option button. */
  readonly radio: { readonly role: "radio"; readonly onClick: () => void };
}

const context = emojiPickerContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const tones: readonly EmojiSkinTone[] = [0, 1, 2, 3, 4, 5];
const options = computed<readonly ToneOption[]>(() =>
  tones.map((tone) => ({
    checked: context.skinTone.value === tone,
    label: labels[tone] ?? `Tone ${tone}`,
    radio: { onClick: () => context.setSkinTone(tone), role: "radio" as const },
    tone,
  })),
);
const groupProps = computed(() => ({ role: "radiogroup" as const, onKeydown }));

function focusTone(tone: EmojiSkinTone): void {
  const radio = element.value?.querySelector<HTMLElement>(`[data-tone="${tone}"]`);
  radio?.focus();
}

function onKeydown(event: KeyboardEvent): void {
  const current = context.skinTone.value;
  let next: number | null = null;
  if (event.key === "ArrowRight" || event.key === "ArrowDown") next = (current + 1) % 6;
  else if (event.key === "ArrowLeft" || event.key === "ArrowUp") next = (current + 5) % 6;
  else if (event.key === "Home") next = 0;
  else if (event.key === "End") next = 5;
  if (next === null) return;
  event.preventDefault();
  const tone = toSkinTone(next);
  context.setSkinTone(tone);
  focusTone(tone);
}
</script>

<template>
  <div
    ref="element"
    v-bind="groupProps"
    :aria-label="ariaLabel"
    data-vize-ui="emoji-picker-skin-tone"
    part="skin-tone"
    :data-skin-tone="context.skinTone.value"
  >
    <button
      v-for="option in options as readonly ToneOption[]"
      :key="option.tone"
      v-bind="option.radio"
      type="button"
      :tabindex="option.checked ? 0 : -1"
      :aria-checked="option.checked ? 'true' : 'false'"
      :aria-label="option.label"
      data-vize-ui="emoji-picker-skin-tone-option"
      part="skin-tone-option"
      :data-tone="option.tone"
      :data-state="option.checked ? 'checked' : 'unchecked'"
    >
      <slot :tone="option.tone" :label="option.label" :checked="option.checked" />
    </button>
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
