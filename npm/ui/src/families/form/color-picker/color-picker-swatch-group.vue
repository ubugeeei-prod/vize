<script setup lang="ts">
import { computed, useTemplateRef, watch } from "vue";

import { createCollectionRegistry } from "../../foundations/collection/collection.ts";
import { useCompositeNavigation } from "../../foundations/composite-navigation/composite-navigation.ts";
import { deriveDeterministicId } from "../../foundations/id/deterministic-id.ts";
import { colorEquals, parseColor } from "./color-picker-color.ts";
import { colorPickerContext } from "./color-picker-context.ts";
import {
  colorPickerSwatchGroupContext,
  getSwatchIdSegment,
} from "./color-picker-swatch-context.ts";
import type { ColorPickerSwatchGroupContextValue } from "./color-picker-swatch-context.ts";
import type { ColorPickerSlotState, ColorPickerSwatchGroupExpose } from "./color-picker-types.ts";

const {
  loop = true,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
} = defineProps<{
  /**
   * Whether arrow-key navigation wraps at the first and last enabled swatch.
   *
   * @default true
   */
  readonly loop?: boolean;

  /**
   * Accessible name of the radiogroup, e.g. `"Saved colors"`.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids that label the radiogroup.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;

  /**
   * Space-separated ids that describe the radiogroup.
   *
   * @default undefined
   */
  readonly ariaDescribedby?: string;
}>();

defineSlots<{
  /** ColorPickerSwatch children. Receives the root color state. */
  default(props: ColorPickerSlotState): unknown;
}>();

const context = colorPickerContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const groupId = computed(() => context.getPartId("swatches"));
const registry = createCollectionRegistry<string, string>({ disabledBehavior: "skip" });
const checkedKey = computed<string | null>(() => {
  for (const item of registry.navigableItems.value) {
    const parsed = parseColor(item.value);
    if (parsed !== null && colorEquals(parsed, context.color.value)) return item.key;
  }
  return null;
});
const slotState = computed<ColorPickerSlotState>(() => ({
  color: context.color.value,
  disabled: context.disabled.value,
  format: context.format.value,
  readOnly: context.readOnly.value,
  state: context.state.value,
  value: context.value.value,
}));

function getSwatchId(value: string): string {
  return deriveDeterministicId(groupId.value, getSwatchIdSegment(value));
}

function syncActiveValue(): void {
  if (context.disabled.value) {
    if (registry.activeKey.value !== null) registry.setActiveKey(null);
    return;
  }
  const target = checkedKey.value ?? registry.navigableItems.value[0]?.key ?? null;
  if (target !== registry.activeKey.value) registry.setActiveKey(target);
}

const navigation = useCompositeNavigation<string, string>({
  registry,
  focusStrategy: "roving",
  getItemId: ({ key }) => getSwatchId(key),
  orientation: "both",
  direction: context.dir,
  loop: () => loop,
  isDisabled: context.disabled,
  onNavigate(change) {
    if (change.intent === "pointer" || !context.editable.value) return;
    const parsed = parseColor(change.key);
    if (parsed === null) return;
    if (context.setColor(parsed, "swatch", change.originalEvent)) {
      context.commit("swatch", change.originalEvent);
    }
  },
});

watch([context.disabled, checkedKey, registry.navigableItems], syncActiveValue, {
  flush: "post",
  immediate: true,
});

colorPickerSwatchGroupContext.provide({
  disabled: context.disabled,
  getSwatchId,
  navigation,
  registry,
  syncActiveValue,
} satisfies ColorPickerSwatchGroupContextValue);

function focus(options?: FocusOptions): void {
  syncActiveValue();
  const key = registry.activeKey.value;
  const item = key === null ? undefined : registry.getItem(key);
  if (item?.element instanceof HTMLElement) item.element.focus(options);
}

const exposed = { element, focus } satisfies {
  readonly element: typeof element;
  readonly focus: ColorPickerSwatchGroupExpose["focus"];
};

defineExpose(exposed);
</script>

<template>
  <div
    :id="groupId"
    ref="element"
    role="radiogroup"
    :aria-label="ariaLabel"
    :aria-labelledby="ariaLabelledby"
    :aria-describedby="ariaDescribedby"
    :aria-disabled="context.disabled.value ? 'true' : undefined"
    :aria-readonly="context.readOnly.value ? 'true' : undefined"
    data-vize-ui="color-picker-swatch-group"
    part="swatch-group"
    :data-state="context.state.value"
    :data-disabled="context.disabled.value ? 'true' : undefined"
  >
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
