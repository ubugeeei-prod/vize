<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import { resolveSurfaceAria } from "./surface-runtime.ts";
import type {
  SurfaceAs,
  SurfaceElement,
  SurfaceElevation,
  SurfaceExpose,
  SurfaceProps,
  SurfaceSlotState,
  SurfaceTone,
} from "./surface-types.ts";

const {
  as = "section",
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
  tone = undefined,
  elevation = undefined,
} = defineProps<SurfaceProps>();

defineSlots<{
  /** Renders surface content with current semantic and data-hook state. */
  default(props: SurfaceSlotState): unknown;
}>();

const element = useTemplateRef<SurfaceElement>("element");
const asState = computed<SurfaceAs>(() => as);
const toneState = computed<SurfaceTone | undefined>(() => tone);
const elevationState = computed<SurfaceElevation | undefined>(() => elevation);
const ariaState = computed(() => resolveSurfaceAria({ ariaDescribedby, ariaLabelledby }));
const ariaLabelledbyState = computed(() => ariaState.value.ariaLabelledby);
const ariaDescribedbyState = computed(() => ariaState.value.ariaDescribedby);
const labelled = computed(() => ariaLabelledbyState.value !== undefined);
const described = computed(() => ariaDescribedbyState.value !== undefined);
const ariaAttributes = computed<{
  readonly "aria-describedby"?: string;
  readonly "aria-labelledby"?: string;
}>(() => {
  const attributes: {
    "aria-describedby"?: string;
    "aria-labelledby"?: string;
  } = {};
  if (ariaDescribedbyState.value !== undefined) {
    attributes["aria-describedby"] = ariaDescribedbyState.value;
  }
  if (ariaLabelledbyState.value !== undefined) {
    attributes["aria-labelledby"] = ariaLabelledbyState.value;
  }
  return attributes;
});
const slotState = computed<SurfaceSlotState>(() => ({
  ariaDescribedby: ariaDescribedbyState.value,
  ariaLabelledby: ariaLabelledbyState.value,
  as: asState.value,
  described: described.value,
  elevation: elevationState.value,
  labelled: labelled.value,
  tone: toneState.value,
}));

type SurfaceSetupExpose = Omit<
  SurfaceExpose,
  | "ariaDescribedby"
  | "ariaLabelledby"
  | "as"
  | "described"
  | "element"
  | "elevation"
  | "labelled"
  | "tone"
> & {
  readonly ariaDescribedby: typeof ariaDescribedbyState;
  readonly ariaLabelledby: typeof ariaLabelledbyState;
  readonly as: typeof asState;
  readonly described: typeof described;
  readonly element: typeof element;
  readonly elevation: typeof elevationState;
  readonly labelled: typeof labelled;
  readonly tone: typeof toneState;
};

const exposed = {
  ariaDescribedby: ariaDescribedbyState,
  ariaLabelledby: ariaLabelledbyState,
  as: asState,
  described,
  element,
  elevation: elevationState,
  labelled,
  tone: toneState,
} satisfies SurfaceSetupExpose;

defineExpose(exposed);
</script>

<template>
  <component
    :is="asState"
    ref="element"
    v-bind="ariaAttributes"
    part="root"
    data-vize-ui="surface"
    :data-tone="toneState"
    :data-elevation="elevationState"
  >
    <slot v-bind="slotState" />
  </component>
</template>

<style scoped>
/* Headless by design. Surface shape, spacing, color, and shadow remain consumer-owned. */
</style>
