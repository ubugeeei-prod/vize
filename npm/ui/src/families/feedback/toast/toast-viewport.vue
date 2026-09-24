<script setup lang="ts" generic="Data = unknown">
import { computed, onMounted, onUnmounted, useTemplateRef, watch } from "vue";

import LiveRegion from "../../accessibility/live-region/live-region.vue";
import type { LiveRegionPoliteness } from "../../accessibility/live-region/live-region-types.ts";
import { toastProviderContext, toastViewportContext } from "./toast-context.ts";
import { formatToastHotkey, matchesToastHotkey } from "./toast-hotkey.ts";
import ToastAction from "./toast-action.vue";
import ToastClose from "./toast-close.vue";
import ToastDescription from "./toast-description.vue";
import ToastRoot from "./toast-root.vue";
import ToastTitle from "./toast-title.vue";
import type { ToastRecord, ToastViewportExpose, ToastViewportSlotState } from "./toast-types.ts";
import { useToast } from "./use-toast.ts";

const {
  pauseOnHover = true,
  pauseOnFocus = true,
  announce = true,
} = defineProps<{
  /**
   * Pause every timer while a pointer hovers the region.
   *
   * @default true
   */
  readonly pauseOnHover?: boolean;

  /**
   * Pause every timer while focus is inside the region.
   *
   * @default true
   */
  readonly pauseOnFocus?: boolean;

  /**
   * Announce new and updated toasts through the embedded live region.
   *
   * @default true
   */
  readonly announce?: boolean;
}>();

defineSlots<{
  /** Renders one visible toast. Falls back to ToastRoot with every part. */
  default?(props: ToastViewportSlotState<Data>): unknown;
}>();

const provider = toastProviderContext.use();
const store = useToast<Data>();
const element = useTemplateRef<HTMLElement>("element");
const announcer = useTemplateRef<InstanceType<typeof LiveRegion>>("announcer");
const regionLabel = computed(
  () => `${provider.label.value} (${formatToastHotkey(provider.hotkey.value)})`,
);
const visible = computed<readonly ToastRecord<Data>[]>(() => store.visibleToasts.value);
const visibleCount = computed<number>(() => store.visibleToasts.value.length);
const announcerStyle = {
  position: "absolute",
  width: "1px",
  height: "1px",
  padding: "0",
  margin: "-1px",
  overflow: "hidden",
  clipPath: "inset(50%)",
  whiteSpace: "nowrap",
  border: "0",
} as const;
const announced = new Map<string, number>();
let ownerDocument: Document | null = null;

function focus(options?: FocusOptions): void {
  element.value?.focus(options);
}

toastViewportContext.provide({ focus });

function announcementText(toast: ToastRecord<Data>): string {
  return [toast.title, toast.description, toast.action?.altText]
    .filter((part): part is string => typeof part === "string" && part.length > 0)
    .join(". ");
}

function announcePending(): void {
  if (!announce || !announcer.value) return;
  const texts: string[] = [];
  let politeness: LiveRegionPoliteness = "polite";
  const liveIds = new Set<string>();
  for (const toast of visible.value) {
    liveIds.add(toast.id);
    if (!toast.open || announced.get(toast.id) === toast.revision) continue;
    announced.set(toast.id, toast.revision);
    const text = announcementText(toast);
    if (text.length === 0) continue;
    texts.push(text);
    if (toast.priority === "high" || toast.type === "error") politeness = "assertive";
  }
  for (const id of announced.keys()) if (!liveIds.has(id)) announced.delete(id);
  if (texts.length > 0) announcer.value.announce(texts.join(". "), politeness);
}

function dismissFor(id: string): () => boolean {
  return () => store.dismiss(id, "api");
}

function onDocumentKeydown(event: KeyboardEvent): void {
  if (!matchesToastHotkey(event, provider.hotkey.value)) return;
  event.preventDefault();
  focus();
}

function onPointerenter(): void {
  if (pauseOnHover) store.pause("hover");
}

function onPointerleave(): void {
  store.resume("hover");
}

function onFocusin(): void {
  if (pauseOnFocus) store.pause("focus");
}

function onFocusout(event: FocusEvent): void {
  const next = event.relatedTarget;
  if (next instanceof Node && element.value?.contains(next)) return;
  store.resume("focus");
}

watch(visible, announcePending, { flush: "post" });

onMounted(() => {
  ownerDocument = globalThis.document;
  ownerDocument.addEventListener("keydown", onDocumentKeydown);
  announcePending();
});

onUnmounted(() => {
  ownerDocument?.removeEventListener("keydown", onDocumentKeydown);
  ownerDocument = null;
  store.resume("hover");
  store.resume("focus");
});

type ToastViewportSetupExpose = Omit<ToastViewportExpose, "element"> & {
  readonly element: typeof element;
};

const exposed = { element, focus } satisfies ToastViewportSetupExpose;

defineExpose(exposed);
</script>

<template>
  <section
    ref="element"
    :aria-label="regionLabel"
    tabindex="-1"
    data-vize-ui="toast-viewport"
    part="viewport"
    :data-paused="store.paused.value ? 'true' : undefined"
    @pointerenter="onPointerenter"
    @pointerleave="onPointerleave"
    @focusin="onFocusin"
    @focusout="onFocusout"
  >
    <ol data-vize-ui="toast-list" part="list">
      <template v-for="(toast, index) in visible as readonly ToastRecord<Data>[]" :key="toast.id">
        <slot :toast :index :count="visibleCount" :dismiss="dismissFor(toast.id)">
          <ToastRoot :toast>
            <ToastTitle v-if="toast.title" />
            <ToastDescription v-if="toast.description" />
            <ToastAction v-if="toast.action" :alt-text="toast.action.altText" />
            <ToastClose v-if="toast.dismissible" />
          </ToastRoot>
        </slot>
      </template>
    </ol>
    <div data-vize-ui="toast-announcer" part="announcer" :style="announcerStyle">
      <LiveRegion ref="announcer" />
    </div>
  </section>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
