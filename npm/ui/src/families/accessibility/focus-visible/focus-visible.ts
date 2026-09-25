/** Uniform focus-visible data attributes driven by the document interaction modality. */
export { default as FocusVisibleProvider } from "./focus-visible-provider.vue";
export { focusVisibleContext } from "./focus-visible-context.ts";
export {
  isTextEntryElement,
  shouldShowFocusRing,
  useFocusVisible,
} from "./focus-visible-runtime.ts";
export type {
  FocusVisibleModality,
  FocusVisibleProviderExpose,
  FocusVisibleSlotState,
  FocusVisibleState,
} from "./focus-visible-types.ts";
