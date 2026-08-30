<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import { useDeterministicId } from "../../../deterministic-id.ts";
import type {
  IconAriaState,
  IconElement,
  IconExpose,
  IconProps,
  IconSize,
  IconSlotState,
} from "./icon-types.ts";

const {
  as = "svg",
  id = undefined,
  viewBox = "0 0 24 24",
  width = "1em",
  height = "1em",
  size = "md",
  focusable = false,
  fill = "none",
  stroke = "currentColor",
  strokeWidth = "2",
  strokeLinecap = "round",
  strokeLinejoin = "round",
  decorative: decorativeProp = undefined,
  ariaHidden = undefined,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
  title = undefined,
  description = undefined,
  titleId = undefined,
  descriptionId = undefined,
} = defineProps<IconProps>();

defineSlots<{
  /** Renders SVG child nodes with current accessibility and styling hooks. */
  default(props: IconSlotState): unknown;
}>();

const element = useTemplateRef<IconElement>("element");
const generatedTitleId = useDeterministicId({ id: () => titleId, hint: "icon-title" });
const generatedDescriptionId = useDeterministicId({
  id: () => descriptionId,
  hint: "icon-description",
});

const sizeValue = computed<IconSize>(() => size);
const hasTitle = computed(() => hasText(title));
const hasDescription = computed(() => hasText(description));
const hasAccessibleName = computed(
  () => hasText(ariaLabel) || hasText(ariaLabelledby) || hasTitle.value,
);
const decorative = computed(
  () => ariaHidden === true || decorativeProp === true || !hasAccessibleName.value,
);
const ariaState = computed<IconAriaState>(() => (decorative.value ? "decorative" : "img"));
const titleIdValue = computed(() =>
  ariaState.value === "img" && hasTitle.value ? generatedTitleId.value : undefined,
);
const descriptionIdValue = computed(() =>
  ariaState.value === "img" && hasDescription.value ? generatedDescriptionId.value : undefined,
);
const ariaLabelValue = computed(() =>
  ariaState.value === "img" && !hasText(ariaLabelledby) && hasText(ariaLabel)
    ? ariaLabel
    : undefined,
);
const ariaLabelledbyValue = computed(() => {
  if (ariaState.value !== "img") return undefined;
  if (hasText(ariaLabelledby)) return ariaLabelledby;
  return hasText(ariaLabel) ? undefined : titleIdValue.value;
});
const ariaDescribedbyValue = computed(() => {
  if (ariaState.value !== "img") return undefined;
  if (hasText(ariaDescribedby)) return ariaDescribedby;
  return descriptionIdValue.value;
});
const slotState = computed<IconSlotState>(() => ({
  ariaState: ariaState.value,
  decorative: decorative.value,
  descriptionId: descriptionIdValue.value,
  size: sizeValue.value,
  titleId: titleIdValue.value,
  viewBox,
}));

type IconSetupExpose = Omit<
  IconExpose,
  "ariaState" | "decorative" | "descriptionId" | "element" | "size" | "titleId" | "viewBox"
> & {
  readonly ariaState: typeof ariaState;
  readonly decorative: typeof decorative;
  readonly descriptionId: typeof descriptionIdValue;
  readonly element: typeof element;
  readonly size: typeof sizeValue;
  readonly titleId: typeof titleIdValue;
  readonly viewBox: string;
};

const exposed = {
  ariaState,
  decorative,
  descriptionId: descriptionIdValue,
  element,
  size: sizeValue,
  titleId: titleIdValue,
  viewBox,
} satisfies IconSetupExpose;

defineExpose(exposed);

function hasText(value: string | undefined): boolean {
  return value != null && value.trim().length > 0;
}
</script>

<template>
  <component
    :is="as"
    :id="id"
    ref="element"
    :role="ariaState === 'img' ? 'img' : undefined"
    :aria-hidden="ariaState === 'decorative' ? 'true' : undefined"
    :aria-label="ariaLabelValue"
    :aria-labelledby="ariaLabelledbyValue"
    :aria-describedby="ariaDescribedbyValue"
    :viewBox="viewBox"
    :width="width"
    :height="height"
    :focusable="focusable ? 'true' : 'false'"
    :fill="fill"
    :stroke="stroke"
    :stroke-width="strokeWidth"
    :stroke-linecap="strokeLinecap"
    :stroke-linejoin="strokeLinejoin"
    data-vize-ui="icon"
    part="root"
    :data-aria-state="ariaState"
    :data-decorative="decorative ? 'true' : 'false'"
    :data-size="sizeValue"
    :data-title="titleIdValue === undefined ? 'missing' : 'present'"
    :data-description="descriptionIdValue === undefined ? 'missing' : 'present'"
  >
    <title v-if="titleIdValue !== undefined" :id="titleIdValue">{{ title }}</title>
    <desc v-if="descriptionIdValue !== undefined" :id="descriptionIdValue">{{ description }}</desc>
    <slot v-bind="slotState" />
  </component>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
