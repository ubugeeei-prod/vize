<script setup lang="ts">
import { computed, onMounted, shallowRef, useTemplateRef, watch } from "vue";
import type { ComputedRef } from "vue";

import { collapsibleContext } from "../collapsible/collapsible.ts";
import type { CollapsibleState } from "../collapsible/collapsible.ts";
import { accordionContext, accordionItemContext } from "./accordion-context.ts";
import type {
  AccordionContentExpose,
  AccordionContentRole,
  AccordionItemSlotState,
  AccordionValue,
} from "./accordion-types.ts";

interface AccordionContentSize {
  readonly height: number;
  readonly width: number;
}

const {
  role = "region",
  hiddenUntilFound = undefined,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
} = defineProps<{
  /**
   * Landmark role for the panel. `null` renders a plain `div`; prefer `null`
   * when an accordion has many panels to avoid landmark proliferation.
   *
   * @default "region"
   */
  readonly role?: AccordionContentRole | null;

  /**
   * Keep this closed panel searchable with `hidden="until-found"`.
   * `undefined` uses the root `hiddenUntilFound`.
   *
   * @default undefined
   */
  readonly hiddenUntilFound?: boolean;

  /**
   * Accessible name when no visible label or `aria-labelledby` supplies one.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids that label the panel. `null` omits the default trigger id.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string | null;

  /**
   * Space-separated ids that describe the panel.
   *
   * @default undefined
   */
  readonly ariaDescribedby?: string;
}>();

defineSlots<{
  /** Panel contents. Receives the item state. */
  default(props: AccordionItemSlotState): unknown;
}>();

const context = accordionContext.use();
const item = accordionItemContext.use();
const collapsible = collapsibleContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const size = shallowRef<AccordionContentSize | null>(null);
const untilFound = computed(() => hiddenUntilFound ?? context.hiddenUntilFound.value);
const roleValue = computed(() => role ?? undefined);
const ariaLabelledbyValue = computed(() => {
  if (roleValue.value === undefined || ariaLabel) return undefined;
  return ariaLabelledby ?? collapsible.triggerId.value;
});
const contentStyle = computed(() =>
  size.value === null
    ? undefined
    : {
        "--vize-accordion-content-height": `${size.value.height}px`,
        "--vize-accordion-content-width": `${size.value.width}px`,
      },
);
let mounted = false;

/**
 * Vue serializes and patches `hidden` as a boolean attribute, which drops the
 * `until-found` keyword. Server and hydration markup therefore use plain
 * `hidden`, and the keyword is applied after mount and after every patch.
 */
function syncHiddenKeyword(): void {
  if (!mounted || collapsible.open.value) return;
  element.value?.setAttribute("hidden", untilFound.value ? "until-found" : "");
}

function measure(): void {
  if (!mounted || !collapsible.open.value || !element.value) return;
  size.value = { height: element.value.scrollHeight, width: element.value.scrollWidth };
}

function onBeforeMatch(event: Event): void {
  collapsible.expand(event);
}

watch(
  [() => collapsible.open.value, untilFound, element],
  () => {
    syncHiddenKeyword();
    measure();
  },
  { flush: "post" },
);

onMounted(() => {
  mounted = true;
  syncHiddenKeyword();
  measure();
});

type AccordionContentSetupExpose = Omit<
  AccordionContentExpose,
  "disabled" | "element" | "hiddenUntilFound" | "locked" | "open" | "state" | "value"
> & {
  readonly disabled: ComputedRef<boolean>;
  readonly element: typeof element;
  readonly hiddenUntilFound: ComputedRef<boolean>;
  readonly locked: ComputedRef<boolean>;
  readonly open: ComputedRef<boolean>;
  readonly state: ComputedRef<CollapsibleState>;
  readonly value: ComputedRef<AccordionValue>;
};

const exposed = {
  disabled: collapsible.disabled,
  element,
  hiddenUntilFound: untilFound,
  locked: item.locked,
  open: collapsible.open,
  state: collapsible.state,
  value: item.value,
} satisfies AccordionContentSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    :id="collapsible.contentId.value"
    ref="element"
    :role="roleValue"
    :hidden="collapsible.open.value ? undefined : true"
    :aria-label="roleValue === undefined ? undefined : ariaLabel"
    :aria-labelledby="ariaLabelledbyValue"
    :aria-describedby="roleValue === undefined ? undefined : ariaDescribedby"
    :style="contentStyle"
    data-vize-ui="accordion-content"
    part="content"
    :data-state="collapsible.state.value"
    :data-orientation="context.orientation.value"
    :data-disabled="collapsible.disabled.value ? 'true' : undefined"
    :data-hidden-until-found="untilFound ? 'true' : undefined"
    @beforematch="onBeforeMatch"
  >
    <slot
      :disabled="collapsible.disabled.value"
      :locked="item.locked.value"
      :open="collapsible.open.value"
      :state="collapsible.state.value"
      :value="item.value.value"
    />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
