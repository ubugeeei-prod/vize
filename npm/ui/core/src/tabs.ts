/** Accessible, unstyled tabset primitive with roving focus and SSR-stable ids. */
export { default as Tabs, default as TabsRoot } from "./tabs-root.vue";
export { default as TabsContent } from "./tabs-content.vue";
export { default as TabsList } from "./tabs-list.vue";
export { default as TabsTrigger } from "./tabs-trigger.vue";
export type {
  TabsActivationMode,
  TabsContentExpose,
  TabsContentSlotState,
  TabsContentState,
  TabsDirection,
  TabsListExpose,
  TabsListSlotState,
  TabsOrientation,
  TabsRootExpose,
  TabsSlotState,
  TabsState,
  TabsTriggerExpose,
  TabsTriggerSlotState,
  TabsTriggerState,
  TabsValue,
} from "./tabs-types.ts";
