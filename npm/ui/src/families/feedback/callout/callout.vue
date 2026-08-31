<script setup lang="ts">
import { computed, useSlots, useTemplateRef } from "vue";

import { useDeterministicId } from "../../foundations/id/deterministic-id.ts";
import {
  normalizeCalloutIdReferenceList,
  normalizeCalloutLabel,
  resolveCalloutAriaState,
  resolveCalloutLive,
} from "./callout-runtime.ts";
import type {
  CalloutAriaState,
  CalloutElement,
  CalloutExpose,
  CalloutProps,
  CalloutSlotState,
  CalloutState,
} from "./callout-types.ts";

const {
  as = "section",
  id = undefined,
  role = "note",
  open = true,
  atomic = true,
  tone = "neutral",
  density = "comfortable",
  iconAriaHidden = true,
  ariaHidden = undefined,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
  titleId = undefined,
  descriptionId = undefined,
} = defineProps<CalloutProps>();

defineSlots<{
  /** Renders the main Callout body. */
  default(props: CalloutSlotState): unknown;

  /** Renders an optional consumer-owned icon. */
  icon(props: CalloutSlotState): unknown;

  /** Renders an optional accessible title. */
  title(props: CalloutSlotState): unknown;

  /** Renders an optional accessible description. */
  description(props: CalloutSlotState): unknown;

  /** Renders optional interactive or navigational actions. */
  actions(props: CalloutSlotState): unknown;
}>();

const slots = useSlots();
const element = useTemplateRef<CalloutElement>("element");
const generatedTitleId = useDeterministicId({ id: () => titleId, hint: "callout-title" });
const generatedDescriptionId = useDeterministicId({
  id: () => descriptionId,
  hint: "callout-description",
});

const state = computed<CalloutState>(() => (open ? "open" : "closed"));
const hasIcon = computed(() => slots.icon !== undefined);
const hasTitle = computed(() => slots.title !== undefined);
const hasDescription = computed(() => slots.description !== undefined);
const hasActions = computed(() => slots.actions !== undefined);
const hasDefault = computed(() => slots.default !== undefined);
const hasContent = computed(
  () => hasTitle.value || hasDescription.value || hasDefault.value || hasActions.value,
);
const ariaState = computed<CalloutAriaState>(() =>
  resolveCalloutAriaState({ ariaHidden: ariaHidden === true || !open, role }),
);
const live = computed(() => resolveCalloutLive(ariaState.value));
const titleIdValue = computed(() => (hasTitle.value ? generatedTitleId.value : undefined));
const descriptionIdValue = computed(() =>
  hasDescription.value ? generatedDescriptionId.value : undefined,
);
const normalizedAriaLabel = computed(() => normalizeCalloutLabel(ariaLabel));
const normalizedAriaLabelledby = computed(() => normalizeCalloutIdReferenceList(ariaLabelledby));
const normalizedAriaDescribedby = computed(() => normalizeCalloutIdReferenceList(ariaDescribedby));
const ariaLabelValue = computed(() => {
  if (ariaState.value === "decorative") return undefined;
  if (normalizedAriaLabelledby.value !== undefined) return undefined;
  return normalizedAriaLabel.value;
});
const ariaLabelledbyValue = computed(() => {
  if (ariaState.value === "decorative") return undefined;
  if (normalizedAriaLabelledby.value !== undefined) return normalizedAriaLabelledby.value;
  if (normalizedAriaLabel.value !== undefined) return undefined;
  return titleIdValue.value;
});
const ariaDescribedbyValue = computed(() => {
  if (ariaState.value === "decorative") return undefined;
  if (normalizedAriaDescribedby.value !== undefined) return normalizedAriaDescribedby.value;
  return descriptionIdValue.value;
});
const slotState = computed<CalloutSlotState>(() => ({
  ariaDescribedby: ariaDescribedbyValue.value,
  ariaLabelledby: ariaLabelledbyValue.value,
  ariaState: ariaState.value,
  atomic,
  density,
  descriptionId: descriptionIdValue.value,
  hasActions: hasActions.value,
  hasDescription: hasDescription.value,
  hasIcon: hasIcon.value,
  hasTitle: hasTitle.value,
  live: live.value,
  open,
  role,
  state: state.value,
  titleId: titleIdValue.value,
  tone,
}));

type CalloutSetupExpose = Omit<
  CalloutExpose,
  | "ariaDescribedby"
  | "ariaLabelledby"
  | "ariaState"
  | "descriptionId"
  | "element"
  | "hasActions"
  | "hasDescription"
  | "hasIcon"
  | "hasTitle"
  | "live"
  | "state"
  | "titleId"
> & {
  readonly ariaDescribedby: typeof ariaDescribedbyValue;
  readonly ariaLabelledby: typeof ariaLabelledbyValue;
  readonly ariaState: typeof ariaState;
  readonly descriptionId: typeof descriptionIdValue;
  readonly element: typeof element;
  readonly hasActions: typeof hasActions;
  readonly hasDescription: typeof hasDescription;
  readonly hasIcon: typeof hasIcon;
  readonly hasTitle: typeof hasTitle;
  readonly live: typeof live;
  readonly state: typeof state;
  readonly titleId: typeof titleIdValue;
};

const exposed = {
  ariaDescribedby: ariaDescribedbyValue,
  ariaLabelledby: ariaLabelledbyValue,
  ariaState,
  get atomic() {
    return atomic;
  },
  get density() {
    return density;
  },
  descriptionId: descriptionIdValue,
  element,
  hasActions,
  hasDescription,
  hasIcon,
  hasTitle,
  live,
  get open() {
    return open;
  },
  get role() {
    return role;
  },
  state,
  titleId: titleIdValue,
  get tone() {
    return tone;
  },
} satisfies CalloutSetupExpose;

defineExpose(exposed);
</script>

<template>
  <component
    :is="as"
    :id="id"
    ref="element"
    part="root"
    :role="ariaState === 'decorative' ? undefined : role"
    :hidden="open ? undefined : true"
    :aria-hidden="ariaState === 'decorative' ? 'true' : undefined"
    :aria-label="ariaLabelValue"
    :aria-labelledby="ariaLabelledbyValue"
    :aria-describedby="ariaDescribedbyValue"
    :aria-live="live"
    :aria-atomic="live === undefined ? undefined : atomic ? 'true' : 'false'"
    data-vize-ui="callout"
    :data-state="state"
    :data-tone="tone"
    :data-density="density"
    :data-aria-state="ariaState"
    :data-live="live ?? 'off'"
    :data-has-icon="hasIcon ? 'true' : 'false'"
    :data-has-title="hasTitle ? 'true' : 'false'"
    :data-has-description="hasDescription ? 'true' : 'false'"
    :data-has-actions="hasActions ? 'true' : 'false'"
  >
    <span
      v-if="hasIcon"
      part="icon"
      data-vize-ui="callout-icon"
      :aria-hidden="iconAriaHidden ? 'true' : undefined"
    >
      <slot name="icon" v-bind="slotState" />
    </span>
    <div v-if="hasContent" part="content" data-vize-ui="callout-content">
      <div v-if="hasTitle" :id="titleIdValue" part="title" data-vize-ui="callout-title">
        <slot name="title" v-bind="slotState" />
      </div>
      <div
        v-if="hasDescription"
        :id="descriptionIdValue"
        part="description"
        data-vize-ui="callout-description"
      >
        <slot name="description" v-bind="slotState" />
      </div>
      <slot v-bind="slotState" />
      <div v-if="hasActions" part="actions" data-vize-ui="callout-actions">
        <slot name="actions" v-bind="slotState" />
      </div>
    </div>
  </component>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
