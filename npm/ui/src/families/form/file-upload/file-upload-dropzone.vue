<script setup lang="ts">
import { computed, onScopeDispose, onWatcherCleanup, shallowRef, useTemplateRef, watch } from "vue";
import type { ComputedRef } from "vue";

import { deriveDeterministicId } from "../../foundations/id/deterministic-id.ts";
import { fileUploadContext } from "./file-upload-context.ts";
import { transferHasFiles } from "./file-upload-transfer.ts";
import type {
  FileUploadDropzoneExpose,
  FileUploadDropzoneSlotState,
  FileUploadDropzoneState,
  FileUploadPasteScope,
} from "./file-upload-types.ts";

const {
  id = undefined,
  openOnClick = true,
  paste = "self",
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
} = defineProps<{
  /**
   * Consumer-owned dropzone id. `null` and `undefined` derive one from the root id.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Treat the dropzone as a button that opens the native picker on click, Enter, and Space.
   * When `false` the dropzone is only a drop and paste target without a focus stop.
   *
   * @default true
   */
  readonly openOnClick?: boolean;

  /**
   * Where clipboard files are accepted: the focused dropzone, the whole document, or nowhere.
   *
   * @default "self"
   */
  readonly paste?: FileUploadPasteScope;

  /**
   * Accessible name when no visible label or `aria-labelledby` supplies one.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids that label the dropzone.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;

  /**
   * Space-separated ids of hints such as accepted types and size limits.
   *
   * @default undefined
   */
  readonly ariaDescribedby?: string;
}>();

defineSlots<{
  /** Dropzone contents. Receives drag and availability state. */
  default(props: FileUploadDropzoneSlotState): unknown;
}>();

const context = fileUploadContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const fallbackId = computed(() => deriveDeterministicId(context.id.value, "dropzone"));
const dropzoneId = computed(() => id ?? fallbackId.value);
const dragDepth = shallowRef(0);
const rejecting = shallowRef(false);
const dragging = computed(() => dragDepth.value > 0);
const disabled = computed(() => context.disabled.value);
const interactive = computed(() => openOnClick && !disabled.value);
const roleValue = computed<"button" | "group">(() => (openOnClick ? "button" : "group"));
const state = computed<FileUploadDropzoneState>(() => {
  if (disabled.value) return "disabled";
  if (!dragging.value) return "idle";
  return rejecting.value ? "rejecting" : "dragging";
});
const slotState = computed<FileUploadDropzoneSlotState>(() => ({
  disabled: disabled.value,
  dragging: dragging.value,
  rejecting: rejecting.value && dragging.value,
  state: state.value,
}));
const INTERACTIVE_DESCENDANT = "a[href], button, input, select, textarea, [role='button']";

function resetDrag(): void {
  dragDepth.value = 0;
  rejecting.value = false;
  context.dragging.value = false;
}

function onDragenter(event: DragEvent): void {
  if (disabled.value || !transferHasFiles(event.dataTransfer)) return;
  event.preventDefault();
  if (dragDepth.value === 0) {
    rejecting.value = context.isTransferRejected(event.dataTransfer);
    context.dragging.value = true;
  }
  dragDepth.value += 1;
}

function onDragover(event: DragEvent): void {
  if (disabled.value || !transferHasFiles(event.dataTransfer)) return;
  event.preventDefault();
  if (event.dataTransfer) event.dataTransfer.dropEffect = "copy";
}

function onDragleave(event: DragEvent): void {
  if (dragDepth.value === 0 || !transferHasFiles(event.dataTransfer)) return;
  dragDepth.value -= 1;
  if (dragDepth.value === 0) resetDrag();
}

function onDrop(event: DragEvent): void {
  if (!transferHasFiles(event.dataTransfer)) return;
  event.preventDefault();
  resetDrag();
  if (disabled.value) return;
  void context.addTransfer(event.dataTransfer, "drop");
}

function onClick(event: MouseEvent): void {
  if (!interactive.value) return;
  const target = event.target;
  if (target instanceof Element && target !== event.currentTarget) {
    const nested = target.closest(INTERACTIVE_DESCENDANT);
    if (nested && nested !== event.currentTarget) return;
  }
  context.openPicker();
}

function onKeydown(event: KeyboardEvent): void {
  if (!interactive.value || event.target !== event.currentTarget) return;
  if (event.key === "Enter") {
    event.preventDefault();
    context.openPicker();
  } else if (event.key === " ") {
    event.preventDefault();
  }
}

function onKeyup(event: KeyboardEvent): void {
  if (!interactive.value || event.target !== event.currentTarget || event.key !== " ") return;
  event.preventDefault();
  context.openPicker();
}

function handlePaste(event: ClipboardEvent): void {
  if (disabled.value || event.defaultPrevented) return;
  const files = event.clipboardData?.files ? Array.from(event.clipboardData.files) : [];
  if (files.length === 0) return;
  event.preventDefault();
  context.addFiles(files, "paste");
}

function onPaste(event: ClipboardEvent): void {
  if (paste === "self") handlePaste(event);
}

watch(
  () => paste,
  (scope) => {
    if (scope !== "document" || typeof document === "undefined") return;
    const target = document;
    target.addEventListener("paste", handlePaste);
    onWatcherCleanup(() => target.removeEventListener("paste", handlePaste));
  },
  { immediate: true },
);

// Listener map keeps one binding for both roles: `button` when clickable, `group` when only a
// drop and paste target.
const listeners = {
  click: onClick,
  dragenter: onDragenter,
  dragleave: onDragleave,
  dragover: onDragover,
  drop: onDrop,
  keydown: onKeydown,
  keyup: onKeyup,
  paste: onPaste,
};

function focus(options?: FocusOptions): void {
  element.value?.focus(options);
}

const releaseFocusFallback = context.setFocusFallback(() =>
  interactive.value ? element.value : null,
);
onScopeDispose(() => {
  releaseFocusFallback();
  if (dragging.value) context.dragging.value = false;
});

type FileUploadDropzoneSetupExpose = Omit<
  FileUploadDropzoneExpose,
  keyof FileUploadDropzoneSlotState | "element" | "id"
> & {
  readonly disabled: ComputedRef<boolean>;
  readonly dragging: ComputedRef<boolean>;
  readonly element: typeof element;
  readonly id: ComputedRef<string>;
  readonly rejecting: ComputedRef<boolean>;
  readonly state: ComputedRef<FileUploadDropzoneState>;
};

const exposed = {
  disabled,
  dragging,
  element,
  focus,
  id: dropzoneId,
  rejecting: computed(() => slotState.value.rejecting),
  state,
} satisfies FileUploadDropzoneSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    :id="dropzoneId"
    ref="element"
    :role="roleValue"
    :tabindex="interactive ? 0 : undefined"
    :aria-disabled="openOnClick && disabled ? 'true' : undefined"
    :aria-label="ariaLabel"
    :aria-labelledby="ariaLabelledby"
    :aria-describedby="ariaDescribedby"
    data-vize-ui="file-upload-dropzone"
    part="dropzone"
    :data-state="state"
    :data-dragging="dragging ? 'true' : undefined"
    :data-drag-reject="slotState.rejecting ? 'true' : undefined"
    :data-disabled="disabled ? 'true' : undefined"
    v-on="listeners"
  >
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
