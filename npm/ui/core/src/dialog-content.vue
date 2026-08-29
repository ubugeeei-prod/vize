<script setup lang="ts">
import { computed, onMounted, onUnmounted, shallowRef, useTemplateRef, watch } from "vue";
import type { ComputedRef } from "vue";

import { createDismissableLayer } from "./dismissable-layer.ts";
import type {
  DismissableLayerDismissEvent,
  DismissableLayerEscapeKeyDownEvent,
  DismissableLayerFocusOutsideEvent,
  DismissableLayerInteractOutsideEvent,
  DismissableLayerPointerDownOutsideEvent,
} from "./dismissable-layer.ts";
import { dialogContext } from "./dialog-context.ts";
import type { DialogAutoFocusEvent, DialogContentExpose, DialogRole } from "./dialog-types.ts";
import type { DialogSlotState, DialogState } from "./dialog-types.ts";
import { createFocusScope } from "./focus-scope.ts";
import { createFocusGuards } from "./focus-guards.ts";
import { createInertOutside } from "./inert-outside.ts";
import { createScrollLock } from "./scroll-lock.ts";

const {
  role = "dialog",
  forceMount = false,
  trapFocus = true,
  autoFocus = true,
  restoreFocus = true,
  inertOutside = true,
  lockScroll = true,
  closeOnEscape = true,
  closeOnPointerDownOutside = true,
  closeOnFocusOutside = true,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
} = defineProps<{
  /** Dialog role announced by assistive technology. @default "dialog" */
  readonly role?: DialogRole;
  /** Keep content mounted while closed. @default false */
  readonly forceMount?: boolean;
  /** Contain focus inside an open modal dialog. @default true */
  readonly trapFocus?: boolean;
  /** Move focus into content when it opens. @default true */
  readonly autoFocus?: boolean;
  /** Restore focus when content closes. @default true */
  readonly restoreFocus?: boolean;
  /** Make outside content inert while the modal dialog is open. @default true */
  readonly inertOutside?: boolean;
  /** Lock document scroll while the modal dialog is open. @default true */
  readonly lockScroll?: boolean;
  /** Let Escape request dismissal. @default true */
  readonly closeOnEscape?: boolean;
  /** Let outside pointer-down request dismissal. @default true */
  readonly closeOnPointerDownOutside?: boolean;
  /** Let outside focus movement request dismissal. @default true */
  readonly closeOnFocusOutside?: boolean;
  /** Accessible name when no visible title supplies one. @default undefined */
  readonly ariaLabel?: string;
  /**
   * Space-separated ids that label the dialog. `null` omits the default title id.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string | null;
  /**
   * Space-separated ids that describe the dialog. `null` omits the default description id.
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
  /** Dialog content. Receives the current open and modal state. */
  default(props: DialogSlotState): unknown;
}>();

const context = dialogContext.use();
const host = useTemplateRef<HTMLDivElement>("host");
const element = useTemplateRef<HTMLDivElement>("element");
const beforeGuard = useTemplateRef<HTMLSpanElement>("beforeGuard");
const afterGuard = useTemplateRef<HTMLSpanElement>("afterGuard");
const ownerDocument = shallowRef<Document | null>(null);
const present = computed(() => context.open.value || forceMount);
const guarded = computed(() => context.open.value && context.modal.value && trapFocus);
const ariaLabelledbyValue = computed(() =>
  ariaLabel ? undefined : (ariaLabelledby ?? context.titleId.value),
);
const ariaDescribedbyValue = computed(() => ariaDescribedby ?? context.descriptionId.value);
const slotState = computed<DialogSlotState>(() => ({
  modal: context.modal.value,
  open: context.open.value,
  state: context.state.value,
}));

const dismissableLayer = createDismissableLayer({
  root: element,
  branches: () =>
    [beforeGuard.value, afterGuard.value].filter((value): value is HTMLSpanElement => !!value),
  enabled: () => context.open.value,
  escapeKey: () => closeOnEscape,
  outsideFocus: () => closeOnFocusOutside,
  outsidePointerDown: () => closeOnPointerDownOutside,
  onEscapeKeyDown: (event) => emit("escape-key-down", event),
  onFocusOutside: (event) => emit("focus-outside", event),
  onInteractOutside: (event) => emit("interact-outside", event),
  onPointerDownOutside: (event) => emit("pointer-down-outside", event),
  onDismiss: (event) => {
    emit("dismiss", event);
    context.close(event.originalEvent);
  },
});
const focusGuards = createFocusGuards({
  root: element,
  enabled: guarded,
  fallbackFocus: () => element.value,
});
const focusScope = createFocusScope({
  root: element,
  autoFocus: () => autoFocus,
  contain: guarded,
  restoreFocus: () => restoreFocus,
  restoreTarget: () => context.triggerElement.value,
  fallbackFocus: () => element.value,
  onMountAutoFocus: (event) => emit("open-auto-focus", event),
  onUnmountAutoFocus: (event) => emit("close-auto-focus", event),
});
const isolation = createInertOutside({
  root: element,
  branches: () => [context.overlayElement.value].filter((value): value is HTMLElement => !!value),
  enabled: () => context.open.value && context.modal.value && inertOutside,
});
const scrollLock = createScrollLock({
  document: ownerDocument,
  enabled: () => context.open.value && context.modal.value && lockScroll,
});

let mounted = false;

function activateControllers(): void {
  scrollLock.activate();
  isolation.activate();
  dismissableLayer.activate();
  focusGuards.activate();
  focusScope.activate();
}

function deactivateControllers(): void {
  dismissableLayer.deactivate();
  focusGuards.deactivate();
  isolation.deactivate();
  scrollLock.deactivate();
  focusScope.deactivate();
}

function syncControllers(): void {
  ownerDocument.value = element.value?.ownerDocument ?? null;
  if (!mounted || !context.open.value || !element.value) {
    deactivateControllers();
    return;
  }
  activateControllers();
}

watch(
  element,
  (next, previous) => {
    if (previous && context.contentElement.value === previous) context.contentElement.value = null;
    if (next) context.contentElement.value = next;
    syncControllers();
  },
  { flush: "post" },
);
watch(() => context.open.value, syncControllers, { flush: "post" });

onMounted(() => {
  mounted = true;
  syncControllers();
});

onUnmounted(() => {
  mounted = false;
  deactivateControllers();
  dismissableLayer.dispose();
  focusGuards.dispose();
  focusScope.dispose();
  isolation.dispose();
  scrollLock.dispose();
  if (context.contentElement.value === element.value) context.contentElement.value = null;
});

function focusContent(options?: FocusOptions): void {
  element.value?.focus(options);
}

type DialogContentSetupExpose = Omit<
  DialogContentExpose,
  "element" | "modal" | "open" | "state"
> & {
  readonly element: typeof element;
  readonly modal: ComputedRef<boolean>;
  readonly open: ComputedRef<boolean>;
  readonly state: ComputedRef<DialogState>;
};

const exposed = {
  element,
  focusContent,
  focusFirst: () => focusScope.focusFirst(),
  modal: context.modal,
  open: context.open,
  state: context.state,
} satisfies DialogContentSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    ref="host"
    data-vize-ui="dialog-content-host"
    part="content-host"
    :hidden="present && context.open.value ? undefined : true"
    :data-state="context.state.value"
  >
    <span
      v-if="guarded"
      ref="beforeGuard"
      v-bind="focusGuards.beforeProps"
      data-vize-ui="dialog-focus-guard"
      part="focus-guard"
    ></span>
    <div
      v-if="present"
      :id="context.contentId.value"
      ref="element"
      v-bind="dismissableLayer.layerProps"
      :role
      tabindex="-1"
      :aria-modal="context.modal.value ? 'true' : undefined"
      :aria-label="ariaLabel"
      :aria-labelledby="ariaLabelledbyValue"
      :aria-describedby="ariaDescribedbyValue"
      data-vize-ui="dialog-content"
      part="content"
      :data-state="context.state.value"
      :data-modal="context.modal.value ? 'true' : 'false'"
      :data-top-layer="dismissableLayer.isTopLayer.value ? 'true' : 'false'"
    >
      <slot v-bind="slotState" />
    </div>
    <span
      v-if="guarded"
      ref="afterGuard"
      v-bind="focusGuards.afterProps"
      data-vize-ui="dialog-focus-guard"
      part="focus-guard"
    ></span>
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
