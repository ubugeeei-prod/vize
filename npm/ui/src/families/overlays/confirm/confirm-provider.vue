<script setup lang="ts">
import { computed, onScopeDispose } from "vue";
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
const dialogId = useDeterministicId({ id: () => id, hint: "confirm" });
// The queue controls open state through the shared controllable-state contract DialogRoot also
// uses; importing it here keeps Confirm's module order identical in root and subpath bundles.
const openState = useControllableState({
  value: () => queue.active.value !== null,
  defaultValue: false,
});
const open = openState.value;
const state = computed<ConfirmState>(() => (open.value ? "pending" : "idle"));
let chosen: string | null = null;

function settle(value: string | null): void {
  const request = queue.active.value;
  if (request === null) return;
  if (queue.settleActive(value)) emit("settle", request, value);
}

const activeRequests = computed<readonly ConfirmRequest[]>(() =>
  queue.active.value === null ? [] : [queue.active.value],
);

function slotPropsFor(request: ConfirmRequest): ConfirmSlotProps {
  return {
    cancel: () => settle(null),
    request,
    resolve: (value: string) => settle(value),
  };
}

function onActionClick(value: string): void {
  chosen = value;
}

function onOpenChange(value: boolean): void {
  if (value) return;
  const pendingChoice = chosen;
  chosen = null;
  settle(pendingChoice);
}

confirmContext.provide(queue.api);
onScopeDispose(queue.dispose);

type ConfirmProviderSetupExpose = Omit<ConfirmProviderExpose, "active" | "pending" | "state"> & {
  readonly active: ComputedRef<ConfirmRequest | null>;
  readonly pending: ComputedRef<number>;
  readonly state: ComputedRef<ConfirmState>;
};

const exposed = {
  active: queue.active,
  cancelAll: queue.cancelAll,
  choose: queue.api.choose,
  confirm: queue.api.confirm,
  pending: queue.pending,
  state,
} satisfies ConfirmProviderSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    data-vize-ui="confirm-provider"
    part="root"
    :data-state="state"
    :data-pending="queue.pending.value > 0 ? String(queue.pending.value) : undefined"
  >
    <slot />
    <DialogRoot :id="dialogId" :open @update:open="onOpenChange">
      <DialogPortal :to :disabled="portalDisabled">
        <DialogOverlay />
        <AlertDialogContent
          v-for="request in activeRequests as readonly ConfirmRequest[]"
          :key="request.id"
          :lock-scroll
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
                v-for="action in request.actions"
                :key="action.value"
                :data-confirm-action="action.value"
                :data-destructive="action.destructive ? 'true' : undefined"
                @click="() => onActionClick(action.value)"
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
