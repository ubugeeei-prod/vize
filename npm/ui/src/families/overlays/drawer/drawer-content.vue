<script setup lang="ts">
import { computed, onMounted, onUnmounted, shallowRef, useTemplateRef, watch } from "vue";
import type { ComputedRef } from "vue";

import { createFocusScope } from "../../accessibility/focus-scope/focus-scope.ts";
import { createScrollLock } from "../../accessibility/scroll-lock/scroll-lock.ts";
import { dialogContext } from "../dialog/dialog.ts";
import { createDismissableLayer } from "../dismissable-layer/dismissable-layer.ts";
import { drawerContext } from "./drawer-context.ts";
import type {
  DrawerAutoFocusEvent,
  DrawerBackdropPointerDownEvent,
  DrawerContentExpose,
  DrawerDismissEvent,
  DrawerDismissReason,
  DrawerEscapeKeyDownEvent,
  DrawerPointerDownOutsideEvent,
  DrawerSide,
  DrawerSlotState,
  DrawerSnapPoint,
  DrawerState,
} from "./drawer-types.ts";

const noDragSelector =
  'input, textarea, select, [contenteditable=""], [contenteditable="true"], [data-vize-drawer-no-drag]';

const {
  forceMount = false,
  autoFocus = true,
  restoreFocus = true,
  lockScroll = true,
  closeOnEscape = true,
  closeOnBackdropPointerDown = true,
  closeOnPointerDownOutside = true,
  dragFromContent = true,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
} = defineProps<{
  /** Render slot contents while the drawer is closed. @default false */
  readonly forceMount?: boolean;
  /** Move focus into the drawer when it opens. @default true */
  readonly autoFocus?: boolean;
  /** Restore focus to the trigger (or previous focus) when the drawer closes. @default true */
  readonly restoreFocus?: boolean;
  /** Lock document scroll while a modal drawer is open. @default true */
  readonly lockScroll?: boolean;
  /** Let Escape (and the native `cancel` event) request dismissal. @default true */
  readonly closeOnEscape?: boolean;
  /** Let a press on the modal `::backdrop` request dismissal. @default true */
  readonly closeOnBackdropPointerDown?: boolean;
  /** Let a press outside a non-modal drawer request dismissal. @default true */
  readonly closeOnPointerDownOutside?: boolean;
  /** Start drag gestures anywhere on the content, not only on DrawerHandle. @default true */
  readonly dragFromContent?: boolean;
  /** Accessible name when no visible title supplies one. @default undefined */
  readonly ariaLabel?: string;
  /**
   * Space-separated ids that label the drawer. `null` omits the default title id.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string | null;
  /**
   * Space-separated ids that describe the drawer. `null` omits the default description id.
   *
   * @default undefined
   */
  readonly ariaDescribedby?: string | null;
}>();

const emit = defineEmits<{
  /** Fired before automatic entry focus is applied. */
  "open-auto-focus": [event: DrawerAutoFocusEvent];
  /** Fired before automatic focus restoration is applied. */
  "close-auto-focus": [event: DrawerAutoFocusEvent];
  /** Fired before Escape requests dismissal. */
  "escape-key-down": [event: DrawerEscapeKeyDownEvent];
  /** Fired before an outside pointer-down requests dismissal of a non-modal drawer. */
  "pointer-down-outside": [event: DrawerPointerDownOutsideEvent];
  /** Fired before a modal `::backdrop` press requests dismissal. */
  "backdrop-pointer-down": [event: DrawerBackdropPointerDownEvent];
  /** Fired after an unprevented dismissal request. */
  dismiss: [event: DrawerDismissEvent];
}>();

defineSlots<{
  /** Drawer content. Receives the current open, snap, and drag state. */
  default(props: DrawerSlotState): unknown;
}>();

const context = dialogContext.use();
const drawer = drawerContext.use();
const element = useTemplateRef<HTMLDialogElement>("element");
const ownerDocument = shallowRef<Document | null>(null);
const mounted = shallowRef(false);
const retained = shallowRef(context.open.value);
const present = computed(() => retained.value || forceMount);
const canDismiss = computed(() => drawer.dismissible.value);
const ariaLabelledbyValue = computed(() =>
  ariaLabel ? undefined : (ariaLabelledby ?? context.titleId.value),
);
const ariaDescribedbyValue = computed(() => ariaDescribedby ?? context.descriptionId.value);
const snapPointToken = computed(() =>
  drawer.activeSnapPoint.value === null ? undefined : String(drawer.activeSnapPoint.value),
);
const styleVars = computed<Readonly<Record<string, string>> | undefined>(() => {
  if (!mounted.value) return undefined;
  const snap = drawer.snapOffset.value;
  const drag = drawer.dragOffset.value;
  return {
    "--vize-drawer-drag-offset": `${drag}px`,
    "--vize-drawer-offset": `${snap + drag}px`,
    "--vize-drawer-snap-offset": `${snap}px`,
  };
});
const slotState = computed<DrawerSlotState>(() => ({
  activeSnapPoint: drawer.activeSnapPoint.value,
  dragging: drawer.dragging.value,
  modal: context.modal.value,
  open: context.open.value,
  side: drawer.side.value,
  state: context.state.value,
}));

let escapeSeen = false;
let escapeTimer: ReturnType<typeof setTimeout> | null = null;
let suppressClick = false;
let syncingNative = false;
let shownModal: boolean | null = null;
let resizeObserver: ResizeObserver | null = null;
let clickCaptureTarget: HTMLDialogElement | null = null;

function dismiss(reason: DrawerDismissReason, originalEvent: Event | null): void {
  emit("dismiss", { originalEvent, reason });
  context.close(originalEvent);
}

const dismissableLayer = createDismissableLayer({
  root: element,
  enabled: () => context.open.value,
  escapeKey: () => closeOnEscape && canDismiss.value,
  outsideFocus: false,
  outsidePointerDown: () => !context.modal.value && closeOnPointerDownOutside && canDismiss.value,
  onEscapeKeyDown: (event) => {
    rememberEscape();
    emit("escape-key-down", event);
  },
  onPointerDownOutside: (event) => emit("pointer-down-outside", event),
  onDismiss: (event) =>
    dismiss(
      event.reason === "escape-key" ? "escape-key" : "pointer-down-outside",
      event.originalEvent,
    ),
});
const focusScope = createFocusScope({
  root: element,
  autoFocus: () => autoFocus,
  contain: false,
  restoreFocus: () => restoreFocus,
  restoreTarget: () => context.triggerElement.value,
  fallbackFocus: () => element.value,
  onMountAutoFocus: (event) => emit("open-auto-focus", event),
  onUnmountAutoFocus: (event) => emit("close-auto-focus", event),
});
const scrollLock = createScrollLock({
  document: ownerDocument,
  enabled: () => context.open.value && context.modal.value && lockScroll,
});

function rememberEscape(): void {
  escapeSeen = true;
  if (escapeTimer !== null) clearTimeout(escapeTimer);
  escapeTimer = setTimeout(() => {
    escapeSeen = false;
    escapeTimer = null;
  }, 0);
}

function closeNative(dialog: HTMLDialogElement): void {
  if (!dialog.open) return;
  syncingNative = true;
  try {
    if (typeof dialog.close === "function") dialog.close();
    else dialog.removeAttribute("open");
  } finally {
    syncingNative = false;
  }
  shownModal = null;
}

function showNative(dialog: HTMLDialogElement, modal: boolean): void {
  if (dialog.open && shownModal === modal) return;
  closeNative(dialog);
  if (!dialog.isConnected) return;
  if (modal && typeof dialog.showModal === "function") dialog.showModal();
  else if (typeof dialog.show === "function") dialog.show();
  else dialog.setAttribute("open", "");
  shownModal = modal;
}

function observeSize(dialog: HTMLDialogElement | null): void {
  resizeObserver?.disconnect();
  resizeObserver = null;
  if (dialog === null || typeof ResizeObserver !== "function") return;
  resizeObserver = new ResizeObserver(() => drawer.measure());
  resizeObserver.observe(dialog);
}

function activateControllers(): void {
  scrollLock.activate();
  dismissableLayer.activate();
  focusScope.activate();
}

function deactivateControllers(): void {
  dismissableLayer.deactivate();
  scrollLock.deactivate();
  focusScope.deactivate();
}

function sync(): void {
  syncDialog(element.value);
}

function syncDialog(dialog: HTMLDialogElement | null): void {
  bindClickCapture(dialog);
  ownerDocument.value = dialog?.ownerDocument ?? null;
  drawer.dialogElement.value = dialog;
  if (!mounted.value || dialog === null) {
    deactivateControllers();
    return;
  }
  if (!context.open.value) {
    // Restore focus while it is still inside the dialog, then close natively and
    // only afterwards release the retained slot contents.
    deactivateControllers();
    observeSize(null);
    closeNative(dialog);
    retained.value = false;
    return;
  }
  showNative(dialog, context.modal.value);
  drawer.measure();
  observeSize(dialog);
  activateControllers();
}

watch(
  () => context.open.value,
  (value) => {
    if (value) retained.value = true;
  },
  { flush: "sync" },
);
watch(element, sync, { flush: "post" });
watch([() => context.open.value, () => context.modal.value], sync, { flush: "post" });

onMounted(() => {
  mounted.value = true;
  sync();
});

onUnmounted(() => {
  mounted.value = false;
  if (element.value) closeNative(element.value);
  observeSize(null);
  deactivateControllers();
  bindClickCapture(null);
  dismissableLayer.dispose();
  focusScope.dispose();
  scrollLock.dispose();
  if (escapeTimer !== null) clearTimeout(escapeTimer);
  if (drawer.dialogElement.value === element.value) drawer.dialogElement.value = null;
});

function onCancel(event: Event): void {
  // The native cancel request is routed through (possibly controlled) Drawer state.
  event.preventDefault();
  if (escapeSeen || !context.open.value) return;
  if (closeOnEscape && canDismiss.value) dismiss("escape-key", event);
}

function onNativeClose(event: Event): void {
  shownModal = null;
  if (syncingNative || !context.open.value) return;
  // Closed by `<form method="dialog">` or another native path: mirror it into state.
  context.close(event);
}

function isOutsideBox(dialog: HTMLDialogElement, event: PointerEvent): boolean {
  const rect = dialog.getBoundingClientRect();
  return (
    event.clientX < rect.left ||
    event.clientX > rect.right ||
    event.clientY < rect.top ||
    event.clientY > rect.bottom
  );
}

function onPointerDown(event: PointerEvent): void {
  suppressClick = false;
  handlePointerDown(event, element.value);
}

function handlePointerDown(event: PointerEvent, dialog: HTMLDialogElement | null): void {
  const target = event.target;
  if (dialog === null || !(target instanceof Element)) return;
  if (target === dialog && context.modal.value && isOutsideBox(dialog, event)) {
    let prevented = false;
    emit("backdrop-pointer-down", {
      get defaultPrevented() {
        return prevented;
      },
      originalEvent: event,
      preventDefault: () => {
        prevented = true;
      },
    });
    if (!prevented && closeOnBackdropPointerDown && canDismiss.value) dismiss("backdrop", event);
    return;
  }
  const fromHandle = target.closest('[data-vize-ui="drawer-handle"]') !== null;
  if (!fromHandle && (!dragFromContent || target.closest(noDragSelector) !== null)) return;
  if (!drawer.startDrag(event)) return;
  try {
    dialog.setPointerCapture(event.pointerId);
  } catch {
    // Pointer capture is best effort; synthetic or already-released pointers cannot be captured.
  }
}

function onPointerMove(event: PointerEvent): void {
  drawer.moveDrag(event);
}

function onPointerUp(event: PointerEvent): void {
  if (drawer.endDrag(event, false)) suppressClick = true;
}

function onPointerCancel(event: PointerEvent): void {
  drawer.endDrag(event, true);
}

function bindClickCapture(dialog: HTMLDialogElement | null): void {
  if (clickCaptureTarget === dialog) return;
  clickCaptureTarget?.removeEventListener("click", onClickCapture, true);
  clickCaptureTarget = dialog;
  dialog?.addEventListener("click", onClickCapture, true);
}

function onClickCapture(event: MouseEvent): void {
  if (!suppressClick) return;
  suppressClick = false;
  event.preventDefault();
  event.stopPropagation();
}

function focusContent(options?: FocusOptions): void {
  element.value?.focus(options);
}

type DrawerContentSetupExpose = Omit<
  DrawerContentExpose,
  keyof DrawerSlotState | "dragOffset" | "element" | "snapOffset"
> & {
  readonly activeSnapPoint: ComputedRef<DrawerSnapPoint | null>;
  readonly dragOffset: typeof drawer.dragOffset;
  readonly dragging: typeof drawer.dragging;
  readonly element: typeof element;
  readonly modal: ComputedRef<boolean>;
  readonly open: ComputedRef<boolean>;
  readonly side: ComputedRef<DrawerSide>;
  readonly snapOffset: ComputedRef<number>;
  readonly state: ComputedRef<DrawerState>;
};

const exposed = {
  activeSnapPoint: drawer.activeSnapPoint,
  dragOffset: drawer.dragOffset,
  dragging: drawer.dragging,
  element,
  focusContent,
  modal: context.modal,
  open: context.open,
  side: drawer.side,
  snapOffset: drawer.snapOffset,
  state: context.state,
} satisfies DrawerContentSetupExpose;

defineExpose(exposed);
</script>

<template>
  <dialog
    :id="context.contentId.value"
    ref="element"
    v-bind="dismissableLayer.layerProps"
    tabindex="-1"
    :aria-label="ariaLabel"
    :aria-labelledby="ariaLabelledbyValue"
    :aria-describedby="ariaDescribedbyValue"
    data-vize-ui="drawer-content"
    part="content"
    :data-state="context.state.value"
    :data-side="drawer.side.value"
    :data-modal="context.modal.value ? 'true' : 'false'"
    :data-dragging="drawer.dragging.value ? 'true' : undefined"
    :data-snap-point="snapPointToken"
    :data-top-layer="dismissableLayer.isTopLayer.value ? 'true' : 'false'"
    :style="styleVars"
    @cancel="onCancel"
    @close="onNativeClose"
    @pointerdown="onPointerDown"
    @pointermove="onPointerMove"
    @pointerup="onPointerUp"
    @pointercancel="onPointerCancel"
  >
    <slot v-if="present" v-bind="slotState" />
  </dialog>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
