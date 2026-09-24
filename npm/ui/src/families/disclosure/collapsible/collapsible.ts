/** Accessible, unstyled disclosure primitive with controlled and uncontrolled state. */
export { default as Collapsible, default as CollapsibleRoot } from "./collapsible-root.vue";
export { default as CollapsibleContent } from "./collapsible-content.vue";
export { default as CollapsibleTrigger } from "./collapsible-trigger.vue";
/** Typed Collapsible context, published so composed families (for example Accordion) can drive Collapsible parts. */
export { collapsibleContext } from "./collapsible-context.ts";
export type { CollapsibleContextValue } from "./collapsible-context.ts";
export type {
  CollapsibleContentExpose,
  CollapsibleContentRole,
  CollapsibleRootExpose,
  CollapsibleSlotState,
  CollapsibleState,
  CollapsibleTriggerExpose,
} from "./collapsible-types.ts";
