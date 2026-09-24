<script setup lang="ts">
import { computed, useTemplateRef } from "vue";
import type { ComputedRef, ShallowRef } from "vue";

import { PopoverContent } from "../popover/popover.ts";
import type { PopoverContentExpose } from "../popover/popover.ts";
import { popconfirmContext } from "./popconfirm-context.ts";
import type {
  PopconfirmContentExpose,
  PopconfirmInitialFocus,
  PopconfirmPlacement,
  PopconfirmSlotState,
  PopconfirmState,
} from "./popconfirm-types.ts";

const {
  title = undefined,
  description = undefined,
  initialFocus = "cancel",
  placement = "top",
  to = "body",
  portalDisabled = false,
} = defineProps<{
  /**
   * Visible question used as the accessible name. The `title` slot overrides it.
   *
   * @default undefined
   */
  readonly title?: string;

  /**
   * Visible consequence text used as the accessible description. The
   * `description` slot overrides it.
   *
   * @default undefined
   */
  readonly description?: string;

  /**
   * Action focused when the confirmation opens. Cancel is the safe default
   * for destructive confirmations.
   *
   * @default "cancel"
   */
  readonly initialFocus?: PopconfirmInitialFocus;

  /**
   * Preferred placement relative to the trigger.
   *
   * @default "top"
   */
  readonly placement?: PopconfirmPlacement;

  /**
   * CSS selector or element the layer is moved into.
   *
   * @default "body"
   */
  readonly to?: string | HTMLElement;

  /**
   * Render in place instead of teleporting.
   *
   * @default false
   */
  readonly portalDisabled?: boolean;
}>();

const slots = defineSlots<{
  /** Visible title. Receives the confirmation state. */
  title?(props: PopconfirmSlotState): unknown;

  /** Visible description. Receives the confirmation state. */
  description?(props: PopconfirmSlotState): unknown;

  /** Actions, normally PopconfirmCancel and PopconfirmConfirm. */
  default?(props: PopconfirmSlotState): unknown;
}>();

const context = popconfirmContext.use();
const popover = useTemplateRef<PopoverContentExpose>("popover");
const element = computed(() => popover.value?.element ?? null);
const hasDescription = computed(() => description !== undefined || slots.description !== undefined);
const slotState = computed<PopconfirmSlotState>(() => ({
  error: context.error.value,
  open: context.open.value,
  pending: context.pending.value,
  state: context.state.value,
}));

function resolveInitialFocus(): HTMLElement | null {
  if (initialFocus === "none") return null;
  const registered =
    initialFocus === "confirm" ? context.confirmElement.value : context.cancelElement.value;
  if (registered) return registered;
  // Actions may mount after the focus scope activates; fall back to their ids.
  const id = initialFocus === "confirm" ? context.confirmId.value : context.cancelId.value;
  return element.value?.ownerDocument.getElementById(id) ?? null;
}

type PopconfirmContentSetupExpose = Omit<
  PopconfirmContentExpose,
  "element" | "error" | "open" | "pending" | "state"
> & {
  readonly element: typeof element;
  readonly error: Readonly<ShallowRef<unknown>>;
  readonly open: ComputedRef<boolean>;
  readonly pending: ComputedRef<boolean>;
  readonly state: ComputedRef<PopconfirmState>;
};

const exposed = {
  element,
  error: context.error,
  open: context.open,
  pending: context.pending,
  state: context.state,
} satisfies PopconfirmContentSetupExpose;

defineExpose(exposed);
</script>

<template>
  <PopoverContent
    ref="popover"
    role="alertdialog"
    :placement
    :to
    :portal-disabled
    :aria-labelledby="context.titleId.value"
    :aria-describedby="hasDescription ? context.descriptionId.value : undefined"
    :close-on-escape="!context.pending.value"
    :close-on-pointer-down-outside="!context.pending.value"
    :close-on-focus-outside="false"
    :auto-focus="initialFocus !== 'none'"
    :initial-focus="resolveInitialFocus"
    data-popconfirm-part="content-host"
  >
    <div
      data-vize-ui="popconfirm-content"
      part="content"
      :aria-busy="context.pending.value ? 'true' : undefined"
      :data-state="context.state.value"
      :data-error="context.error.value === null ? undefined : 'true'"
    >
      <p :id="context.titleId.value" part="title">
        <slot name="title" v-bind="slotState">{{ title }}</slot>
      </p>
      <p v-if="hasDescription" :id="context.descriptionId.value" part="description">
        <slot name="description" v-bind="slotState">{{ description }}</slot>
      </p>
      <slot v-bind="slotState" />
    </div>
  </PopoverContent>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
