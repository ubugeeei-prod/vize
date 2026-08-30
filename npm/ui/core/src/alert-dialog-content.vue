<script setup lang="ts">
import { computed, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import type { AlertDialogContentExpose, AlertDialogSlotState } from "./alert-dialog-types.ts";
import type {
  DismissableLayerDismissEvent,
  DismissableLayerEscapeKeyDownEvent,
  DismissableLayerFocusOutsideEvent,
  DismissableLayerInteractOutsideEvent,
  DismissableLayerPointerDownOutsideEvent,
} from "./dismissable-layer.ts";
import DialogContent from "./families/overlays/dialog/dialog-content.vue";
import { dialogContext } from "./families/overlays/dialog/dialog-context.ts";
import type { DialogAutoFocusEvent } from "./families/overlays/dialog/dialog-types.ts";

const {
  forceMount = false,
  trapFocus = true,
  autoFocus = true,
  restoreFocus = true,
  inertOutside = true,
  lockScroll = true,
  closeOnEscape = true,
  closeOnPointerDownOutside = false,
  closeOnFocusOutside = false,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
} = defineProps<{
  /** Keep content mounted while closed. @default false */
  readonly forceMount?: boolean;
  /** Contain focus inside an open modal alert dialog. @default true */
  readonly trapFocus?: boolean;
  /** Move focus into content when it opens. @default true */
  readonly autoFocus?: boolean;
  /** Restore focus when content closes. @default true */
  readonly restoreFocus?: boolean;
  /** Make outside content inert while the modal alert dialog is open. @default true */
  readonly inertOutside?: boolean;
  /** Lock document scroll while the modal alert dialog is open. @default true */
  readonly lockScroll?: boolean;
  /** Let Escape request dismissal. @default true */
  readonly closeOnEscape?: boolean;
  /** Let outside pointer-down request dismissal. @default false */
  readonly closeOnPointerDownOutside?: boolean;
  /** Let outside focus movement request dismissal. @default false */
  readonly closeOnFocusOutside?: boolean;
  /** Accessible name when no visible title supplies one. @default undefined */
  readonly ariaLabel?: string;
  /**
   * Space-separated ids that label the alert dialog. `null` omits the default title id.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string | null;
  /**
   * Space-separated ids that describe the alert dialog. `null` omits the default description id.
   *
   * @default undefined
   */
  readonly ariaDescribedby?: string | null;
}>();

const emit = defineEmits<{
  /** Fired before automatic entry focus is applied. */
  "open-auto-focus": [event: DialogAutoFocusEvent];
  /** Fired before automatic focus restoration is applied. */
  "close-auto-focus": [event: DialogAutoFocusEvent];
  /** Fired before Escape requests dismissal. */
  "escape-key-down": [event: DismissableLayerEscapeKeyDownEvent];
  /** Fired before an outside pointer-down requests dismissal. */
  "pointer-down-outside": [event: DismissableLayerPointerDownOutsideEvent];
  /** Fired before outside focus movement requests dismissal. */
  "focus-outside": [event: DismissableLayerFocusOutsideEvent];
  /** Fired before outside pointer or focus interaction requests dismissal. */
  "interact-outside": [event: DismissableLayerInteractOutsideEvent];
  /** Fired after an unprevented dismissal request. */
  dismiss: [event: DismissableLayerDismissEvent];
}>();

defineSlots<{
  /** AlertDialog content. Receives the current open and modal state. */
  default(props: AlertDialogSlotState): unknown;
}>();

const context = dialogContext.use();
const content = useTemplateRef<AlertDialogContentExpose>("content");
const element = computed(() => content.value?.element ?? null);
const open = computed(() => context.open.value);
const modal = computed(() => context.modal.value);
const state = computed(() => context.state.value);
const slotState = computed<AlertDialogSlotState>(() => ({
  modal: modal.value,
  open: open.value,
  state: state.value,
}));

function focusContent(options?: FocusOptions): void {
  content.value?.focusContent(options);
}

type AlertDialogContentSetupExpose = Omit<
  AlertDialogContentExpose,
  "element" | "modal" | "open" | "state"
> & {
  readonly element: ComputedRef<AlertDialogContentExpose["element"]>;
  readonly modal: ComputedRef<AlertDialogContentExpose["modal"]>;
  readonly open: ComputedRef<AlertDialogContentExpose["open"]>;
  readonly state: ComputedRef<AlertDialogContentExpose["state"]>;
};

const exposed = {
  element,
  focusContent,
  focusFirst: () => content.value?.focusFirst() ?? null,
  modal,
  open,
  state,
} satisfies AlertDialogContentSetupExpose;

defineExpose(exposed);

function onOpenAutoFocus(event: DialogAutoFocusEvent): void {
  emit("open-auto-focus", event);
}

function onCloseAutoFocus(event: DialogAutoFocusEvent): void {
  emit("close-auto-focus", event);
}

function onEscapeKeyDown(event: DismissableLayerEscapeKeyDownEvent): void {
  emit("escape-key-down", event);
}

function onPointerDownOutside(event: DismissableLayerPointerDownOutsideEvent): void {
  emit("pointer-down-outside", event);
}

function onFocusOutside(event: DismissableLayerFocusOutsideEvent): void {
  emit("focus-outside", event);
}

function onInteractOutside(event: DismissableLayerInteractOutsideEvent): void {
  emit("interact-outside", event);
}

function onDismiss(event: DismissableLayerDismissEvent): void {
  emit("dismiss", event);
}
</script>

<template>
  <div
    data-vize-ui="alert-dialog-content"
    part="root"
    :data-state="state"
    :data-modal="modal ? 'true' : 'false'"
  >
    <DialogContent
      ref="content"
      role="alertdialog"
      :force-mount
      :trap-focus
      :auto-focus
      :restore-focus
      :inert-outside
      :lock-scroll
      :close-on-escape
      :close-on-pointer-down-outside
      :close-on-focus-outside
      :aria-label
      :aria-labelledby
      :aria-describedby
      @open-auto-focus="onOpenAutoFocus"
      @close-auto-focus="onCloseAutoFocus"
      @escape-key-down="onEscapeKeyDown"
      @pointer-down-outside="onPointerDownOutside"
      @focus-outside="onFocusOutside"
      @interact-outside="onInteractOutside"
      @dismiss="onDismiss"
    >
      <slot v-bind="slotState" />
    </DialogContent>
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
