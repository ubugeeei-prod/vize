/** Headless floating action button and speed-dial menu (WAI-ARIA menu button pattern). */
export { default as FloatingActionButton } from "./floating-action-button.vue";
export { default as SpeedDial, default as SpeedDialRoot } from "./speed-dial-root.vue";
export { default as SpeedDialAction } from "./speed-dial-action.vue";
export { default as SpeedDialContent } from "./speed-dial-content.vue";
export { default as SpeedDialTrigger } from "./speed-dial-trigger.vue";
export type {
  FloatingActionButtonExpose,
  FloatingActionButtonPlacement,
  FloatingActionButtonSlotState,
  SpeedDialActionExpose,
  SpeedDialActionSlotState,
  SpeedDialChangeReason,
  SpeedDialContentExpose,
  SpeedDialDirection,
  SpeedDialRootExpose,
  SpeedDialSelectEvent,
  SpeedDialSlotState,
  SpeedDialState,
  SpeedDialTriggerExpose,
} from "./floating-action-button-types.ts";
