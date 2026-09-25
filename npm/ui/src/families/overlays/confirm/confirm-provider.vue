<script setup lang="ts">
import { computed, nextTick, onScopeDispose, shallowRef, useTemplateRef, watch } from "vue";
import type { ComputedRef } from "vue";

import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import { useDeterministicId } from "../../foundations/id/deterministic-id.ts";
// Import through the AlertDialog entry so Confirm shares its packaged chunks.
import {
  AlertDialogContent,
  AlertDialogAction as DialogClose,
  AlertDialogDescription as DialogDescription,
  AlertDialogOverlay as DialogOverlay,
  AlertDialogPortal as DialogPortal,
  AlertDialogRoot as DialogRoot,
  AlertDialogTitle as DialogTitle,
} from "../alert-dialog/alert-dialog.ts";
import { confirmContext, createConfirmQueue } from "./confirm-runtime.ts";
import type {
  ConfirmApi,
  ConfirmProviderExpose,
  ConfirmRequest,
  ConfirmSlotProps,
  ConfirmState,
} from "./confirm-types.ts";

const {
  id = undefined,
  confirmLabel = "Confirm",
  cancelLabel = "Cancel",
  to = "body",
  portalDisabled = false,
  lockScroll = true,
} = defineProps<{
  /**
   * Consumer-owned base id of the rendered alert dialog. `null` and `undefined` use a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Default label of the accepting action for `confirm()` requests.
   *
   * @default "Confirm"
   */
  readonly confirmLabel?: string;

  /**
   * Default label of the cancelling action.
   *
   * @default "Cancel"
   */
  readonly cancelLabel?: string;

  /**
   * CSS selector or element the alert dialog layer is moved into.
   *
   * @default "body"
   */
  readonly to?: string | HTMLElement;

  /**
   * Render the alert dialog in place instead of teleporting it.
   *
   * @default false
   */
  readonly portalDisabled?: boolean;

  /**
   * Lock document scroll while a request is on screen.
   *
   * @default true
   */
  readonly lockScroll?: boolean;
}>();

const emit = defineEmits<{
  /** Fired after a request settles with its chosen action value, or `null` when cancelled. */
  settle: [request: ConfirmRequest, value: string | null];
}>();

defineSlots<{
  /** Application subtree that may call `useConfirm()`. */
  default?(): unknown;

  /** Replace the alert dialog body for the active request. */
  content?(props: ConfirmSlotProps): unknown;
}>();

const queue = createConfirmQueue({
  cancelLabel: () => cancelLabel,
  confirmLabel: () => confirmLabel,
});
const host = useTemplateRef<HTMLDivElement>("host");
const dialogId = useDeterministicId({ id: () => id, hint: "confirm" });
// The queue controls open state through the shared controllable-state contract DialogRoot also
// uses; importing it here keeps Confirm's module order identical in root and subpath bundles.
const openState = useControllableState({
  value: () => queue.active.value !== null,
  defaultValue: false,
});
const open = openState.value;
const state = computed<ConfirmState>(() => (open.value ? "pending" : "idle"));
const retiring = shallowRef<readonly ConfirmRequest[]>([]);
const presentedRequests = computed<readonly ConfirmRequest[]>(() => {
  const active = queue.active.value;
  return active === null ? retiring.value : retiring.value.concat(active);
});
let chosen: string | null = null;
let restoreTarget: HTMLElement | null = null;
let disposed = false;

watch(
  queue.active,
  (active, previous) => {
    if (active !== null && previous === null && restoreTarget === null) {
      const focused = typeof document === "undefined" ? null : document.activeElement;
      if (
        typeof HTMLElement !== "undefined" &&
        focused instanceof HTMLElement &&
        focused !== document.body
      ) {
        restoreTarget = focused;
      }
    }
    if (active !== null || previous === null) return;
    void nextTick(() => {
      void nextTick(() => {
        if (disposed || queue.active.value !== null) return;
        const target = restoreTarget?.isConnected ? restoreTarget : host.value;
        target?.focus();
        restoreTarget = null;
      });
    });
  },
  { flush: "sync" },
);

function settle(value: string | null, requestId?: string): void {
  const request = queue.active.value;
  if (request === null || (requestId !== undefined && request.id !== requestId)) return;
  if (!queue.settleActive(value)) return;
  retiring.value = retiring.value.concat(request);
  emit("settle", request, value);
}

function cancelAll(): void {
  const request = queue.active.value;
  queue.cancelAll();
  if (request !== null) retiring.value = retiring.value.concat(request);
}

function completeRetirement(requestId: string): void {
  retiring.value = retiring.value.filter((request) => request.id !== requestId);
}

function slotPropsFor(request: ConfirmRequest): ConfirmSlotProps {
  return {
    cancel: () => settle(null, request.id),
    request,
    resolve: (value: string) => settle(value, request.id),
  };
}

interface ConfirmActionButton {
  readonly value: string;
  readonly label: string;
  readonly destructive: boolean;
  readonly onClick: () => void;
}

function actionsOf(request: ConfirmRequest): readonly ConfirmActionButton[] {
  return request.actions.map((action) => ({
    destructive: action.destructive === true,
    label: action.label,
    onClick: () => onActionClick(action.value, request.id),
    value: action.value,
  }));
}

function onActionClick(value: string, requestId: string): void {
  if (queue.active.value?.id !== requestId) return;
  chosen = value;
}

function onOpenChange(value: boolean, requestId: string): void {
  if (value || queue.active.value?.id !== requestId) return;
  const pendingChoice = chosen;
  chosen = null;
  settle(pendingChoice, requestId);
}

const api: ConfirmApi = { ...queue.api, cancelAll };
confirmContext.provide(api);
onScopeDispose(() => {
  disposed = true;
  queue.dispose();
});

type ConfirmProviderSetupExpose = Omit<ConfirmProviderExpose, "active" | "pending" | "state"> & {
  readonly active: ComputedRef<ConfirmRequest | null>;
  readonly pending: ComputedRef<number>;
  readonly state: ComputedRef<ConfirmState>;
};

const exposed = {
  active: queue.active,
  cancelAll,
  choose: queue.api.choose,
  confirm: queue.api.confirm,
  pending: queue.pending,
  state,
} satisfies ConfirmProviderSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    ref="host"
    data-vize-ui="confirm-provider"
    part="root"
    tabindex="-1"
    :data-state="state"
    :data-pending="queue.pending.value > 0 ? String(queue.pending.value) : undefined"
  >
    <slot />
    <DialogRoot
      v-for="request in presentedRequests"
      :id="queue.active.value?.id === request.id ? dialogId : `${dialogId}-${request.id}`"
      :key="request.id"
      :open="queue.active.value?.id === request.id"
      @update:open="(value: boolean) => onOpenChange(value, request.id)"
      @exit-complete="() => completeRetirement(request.id)"
    >
      <DialogPortal :to :disabled="portalDisabled">
        <DialogOverlay />
        <AlertDialogContent
          :lock-scroll
          :restore-focus="false"
          :aria-describedby="request.description === null ? null : undefined"
          :data-confirm-kind="request.kind"
          :data-destructive="request.destructive ? 'true' : undefined"
        >
          <slot name="content" v-bind="slotPropsFor(request)">
            <DialogTitle>{{ request.title }}</DialogTitle>
            <DialogDescription v-if="request.description !== null">
              {{ request.description }}
            </DialogDescription>
            <div data-vize-ui="confirm-actions" part="actions">
              <DialogClose data-confirm-action="cancel">
                {{ request.cancelLabel }}
              </DialogClose>
              <DialogClose
                v-for="action in actionsOf(request)"
                :key="action.value"
                :data-confirm-action="action.value"
                :data-destructive="action.destructive ? 'true' : undefined"
                @click="action.onClick"
              >
                {{ action.label }}
              </DialogClose>
            </div>
          </slot>
        </AlertDialogContent>
      </DialogPortal>
    </DialogRoot>
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
