<script setup lang="ts">
import { computed, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { deriveDeterministicId, useDeterministicId } from "../../../deterministic-id.ts";
import { normalizeBannerAria } from "./banner-aria.ts";
import type {
  BannerElement,
  BannerExpose,
  BannerProps,
  BannerRole,
  BannerSlotState,
  BannerState,
  BannerTone,
} from "./banner-types.ts";

const {
  as = "section",
  id = undefined,
  title = undefined,
  description = undefined,
  role: roleProp = "region",
  tone: toneProp = "neutral",
  open = true,
  dismissible: dismissibleProp = false,
  dismissLabel = "Dismiss banner",
  atomic = true,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
} = defineProps<BannerProps>();
const slots = defineSlots<{
  /** Render the primary banner body. */
  default?(props: BannerSlotState): unknown;

  /** Render a consumer-owned title inside the deterministic title node. */
  title?(props: BannerSlotState): unknown;

  /** Render a consumer-owned description inside the deterministic description node. */
  description?(props: BannerSlotState): unknown;

  /** Render trailing actions. */
  actions?(props: BannerSlotState): unknown;
}>();
const emit = defineEmits<{
  /** Request that the controlled `open` prop moves to a new value. */
  "update:open": [open: boolean];

  /** Fired after the dismiss control is activated. */
  dismiss: [nativeEvent: MouseEvent];
}>();

const element = useTemplateRef<BannerElement>("element");
const rootId = useDeterministicId({ id: () => id, hint: "banner" });
const titleId = computed(() => deriveDeterministicId(rootId.value, "title"));
const descriptionId = computed(() => deriveDeterministicId(rootId.value, "description"));
const role = computed<BannerRole>(() => roleProp);
const tone = computed<BannerTone>(() => toneProp);
const openValue = computed<boolean>(() => open);
const state = computed<BannerState>(() => (openValue.value ? "open" : "closed"));
const dismissible = computed<boolean>(() => dismissibleProp && openValue.value);
const dismissLabelValue = computed(() => normalizeText(dismissLabel) ?? "Dismiss banner");
const hasTitlePart = computed<boolean>(() => hasText(title) || slots.title !== undefined);
const hasDescriptionPart = computed<boolean>(
  () => hasText(description) || slots.description !== undefined,
);
const aria = computed(() =>
  normalizeBannerAria({
    ariaDescribedby,
    ariaLabel,
    ariaLabelledby,
    atomic,
    descriptionId: descriptionId.value,
    hasDescription: hasDescriptionPart.value,
    hasTitle: hasTitlePart.value,
    role: role.value,
    titleId: titleId.value,
  }),
);
const ariaRole = computed(() => aria.value.role);
const ariaLabelValue = computed(() => aria.value.ariaLabel);
const ariaLabelledbyValue = computed(() => aria.value.ariaLabelledby);
const ariaDescribedbyValue = computed(() => aria.value.ariaDescribedby);
const ariaLiveValue = computed(() => aria.value.ariaLive);
const ariaAtomicValue = computed(() => aria.value.ariaAtomic);
const live = computed(() => aria.value.live);
const named = computed(() => aria.value.named);
const ariaState = computed(() => aria.value.ariaState);
const renderedAriaRole = computed(() => (openValue.value ? ariaRole.value : undefined));
const renderedAriaLabel = computed(() => (openValue.value ? ariaLabelValue.value : undefined));
const renderedAriaLabelledby = computed(() =>
  openValue.value ? ariaLabelledbyValue.value : undefined,
);
const renderedAriaDescribedby = computed(() =>
  openValue.value ? ariaDescribedbyValue.value : undefined,
);
const renderedAriaLive = computed(() => (openValue.value ? ariaLiveValue.value : undefined));
const renderedAriaAtomic = computed(() => (openValue.value ? ariaAtomicValue.value : undefined));
const ariaHidden = computed(() => (openValue.value ? undefined : "true"));
const slotState = computed<BannerSlotState>(() => ({
  ariaDescribedby: ariaDescribedbyValue.value,
  ariaLabelledby: ariaLabelledbyValue.value,
  ariaState: ariaState.value,
  descriptionId: descriptionId.value,
  dismissible: dismissible.value,
  live: live.value,
  named: named.value,
  role: role.value,
  state: state.value,
  titleId: titleId.value,
  tone: tone.value,
}));

function hasText(value: string | undefined): boolean {
  return normalizeText(value) !== undefined;
}

function normalizeText(value: string | undefined): string | undefined {
  if (value === undefined) return undefined;
  const normalized = value.replaceAll(/\s+/g, " ").trim();
  return normalized.length === 0 ? undefined : normalized;
}

function focus(options?: FocusOptions): void {
  if (!openValue.value) return;
  focusTarget(element.value, options);
}

function focusTarget(target: unknown, options?: FocusOptions): void {
  if (
    typeof target === "object" &&
    target !== null &&
    "focus" in target &&
    typeof target.focus === "function"
  ) {
    target.focus(options);
  }
}

function dismiss(nativeEvent?: MouseEvent): void {
  emit("update:open", false);
  if (nativeEvent !== undefined) emit("dismiss", nativeEvent);
}

function onDismiss(event: MouseEvent): void {
  dismiss(event);
}

type BannerSetupExpose = Omit<
  BannerExpose,
  | "ariaDescribedby"
  | "ariaLabelledby"
  | "ariaState"
  | "descriptionId"
  | "dismissible"
  | "element"
  | "live"
  | "named"
  | "role"
  | "state"
  | "titleId"
  | "tone"
> & {
  readonly ariaDescribedby: ComputedRef<string | undefined>;
  readonly ariaLabelledby: ComputedRef<string | undefined>;
  readonly ariaState: ComputedRef<BannerSlotState["ariaState"]>;
  readonly descriptionId: ComputedRef<string>;
  readonly dismissible: ComputedRef<boolean>;
  readonly element: typeof element;
  readonly live: ComputedRef<BannerSlotState["live"]>;
  readonly named: ComputedRef<boolean>;
  readonly role: ComputedRef<BannerRole>;
  readonly state: ComputedRef<BannerState>;
  readonly titleId: ComputedRef<string>;
  readonly tone: ComputedRef<BannerTone>;
};

const exposed = {
  ariaDescribedby: ariaDescribedbyValue,
  ariaLabelledby: ariaLabelledbyValue,
  ariaState,
  descriptionId,
  dismiss,
  dismissible,
  element,
  focus,
  live,
  named,
  role,
  state,
  titleId,
  tone,
} satisfies BannerSetupExpose;

defineExpose(exposed);
</script>

<template>
  <component
    :is="as"
    :id="rootId"
    ref="element"
    :hidden="!openValue"
    :role="renderedAriaRole"
    :aria-hidden="ariaHidden"
    :aria-label="renderedAriaLabel"
    :aria-labelledby="renderedAriaLabelledby"
    :aria-describedby="renderedAriaDescribedby"
    :aria-live="renderedAriaLive"
    :aria-atomic="renderedAriaAtomic"
    data-vize-ui="banner"
    part="root"
    :data-state="state"
    :data-tone="tone"
    :data-role="role"
    :data-live="live"
    :data-named="named ? 'true' : 'false'"
    :data-aria-state="ariaState"
    :data-dismissible="dismissible ? 'true' : undefined"
  >
    <div v-if="hasTitlePart" :id="titleId" data-vize-ui="banner-title" part="title">
      <slot name="title" v-bind="slotState">
        {{ title }}
      </slot>
    </div>
    <div
      v-if="hasDescriptionPart"
      :id="descriptionId"
      data-vize-ui="banner-description"
      part="description"
    >
      <slot name="description" v-bind="slotState">
        {{ description }}
      </slot>
    </div>
    <slot v-bind="slotState" />
    <slot name="actions" v-bind="slotState" />
    <button
      v-if="dismissible"
      type="button"
      data-vize-ui="banner-dismiss"
      part="dismiss"
      :aria-label="dismissLabelValue"
      @click="onDismiss"
    >
      {{ dismissLabelValue }}
    </button>
  </component>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
