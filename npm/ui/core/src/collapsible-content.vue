<script setup lang="ts">
import { computed, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { collapsibleContext } from "./collapsible-context.ts";
import type {
  CollapsibleContentExpose,
  CollapsibleContentRole,
  CollapsibleSlotState,
  CollapsibleState,
} from "./collapsible-types.ts";

const {
  role = "region",
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
} = defineProps<{
  /**
   * Optional landmark role for the content. `null` renders a plain `div`.
   *
   * @default "region"
   */
  readonly role?: CollapsibleContentRole | null;

  /**
   * Accessible name when no visible label or `aria-labelledby` supplies one.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids that label the content. `null` omits the default trigger id.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string | null;

  /**
   * Space-separated ids that describe the content.
   *
   * @default undefined
   */
  readonly ariaDescribedby?: string;
}>();

defineSlots<{
  /** Collapsible content. Receives the current open and disabled state. */
  default(props: CollapsibleSlotState): unknown;
}>();

const context = collapsibleContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const roleValue = computed(() => role ?? undefined);
const ariaLabelledbyValue = computed(() => {
  if (roleValue.value === undefined || ariaLabel) return undefined;
  return ariaLabelledby ?? context.triggerId.value;
});

function focusContent(options?: FocusOptions): void {
  element.value?.focus(options);
}

type CollapsibleContentSetupExpose = Omit<
  CollapsibleContentExpose,
  "disabled" | "element" | "open" | "state"
> & {
  readonly disabled: ComputedRef<boolean>;
  readonly element: typeof element;
  readonly open: ComputedRef<boolean>;
  readonly state: ComputedRef<CollapsibleState>;
};

const exposed = {
  disabled: context.disabled,
  element,
  focusContent,
  open: context.open,
  state: context.state,
} satisfies CollapsibleContentSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    :id="context.contentId.value"
    ref="element"
    :role="roleValue"
    :hidden="context.open.value ? undefined : true"
    :aria-label="roleValue === undefined ? undefined : ariaLabel"
    :aria-labelledby="ariaLabelledbyValue"
    :aria-describedby="roleValue === undefined ? undefined : ariaDescribedby"
    data-vize-ui="collapsible-content"
    part="content"
    :data-state="context.state.value"
    :data-disabled="context.disabled.value ? 'true' : undefined"
  >
    <slot
      :disabled="context.disabled.value"
      :open="context.open.value"
      :state="context.state.value"
    />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
