/** Promise-based confirmation helper rendered through the AlertDialog primitive. */
export { default as ConfirmProvider } from "./confirm-provider.vue";
export {
  confirmActionValue,
  confirmContext,
  createConfirmQueue,
  useConfirm,
} from "./confirm-runtime.ts";
export type { ConfirmQueue, ConfirmQueueDefaults } from "./confirm-runtime.ts";
export type {
  ConfirmAction,
  ConfirmApi,
  ConfirmChooseOptions,
  ConfirmOptions,
  ConfirmProviderExpose,
  ConfirmRequest,
  ConfirmRequestKind,
  ConfirmSlotProps,
  ConfirmState,
} from "./confirm-types.ts";
