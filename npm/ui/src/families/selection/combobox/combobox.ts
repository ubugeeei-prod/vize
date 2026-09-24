/**
 * Accessible, unstyled Combobox: an editable APG combobox with a typed listbox popup.
 *
 * Popup structure is shared with Select: `ComboboxContent`, `ComboboxItem`,
 * `ComboboxGroup`, `ComboboxLabel`, `ComboboxSeparator`, `ComboboxViewport`,
 * `ComboboxItemIndicator`, `ComboboxScrollButton`, and `ComboboxVirtualizer`
 * are the Select parts, which publish `data-vize-ui="combobox-*"` inside a
 * `ComboboxRoot`.
 */
export { default as Combobox, default as ComboboxRoot } from "./combobox-root.vue";
export { default as ComboboxAnchor } from "./combobox-anchor.vue";
export { default as ComboboxChip } from "./combobox-chip.vue";
export { default as ComboboxChipRemove } from "./combobox-chip-remove.vue";
export { default as ComboboxCreateItem } from "./combobox-create-item.vue";
export { default as ComboboxEmpty } from "./combobox-empty.vue";
export { default as ComboboxInput } from "./combobox-input.vue";
export { default as ComboboxLoading } from "./combobox-loading.vue";
export { default as ComboboxTrigger } from "./combobox-trigger.vue";
export { default as ComboboxContent } from "../select/select-content.vue";
export { default as ComboboxGroup } from "../select/select-group.vue";
export { default as ComboboxItem } from "../select/select-item.vue";
export { default as ComboboxItemIndicator } from "../select/select-item-indicator.vue";
export { default as ComboboxLabel } from "../select/select-label.vue";
export { default as ComboboxScrollButton } from "../select/select-scroll-button.vue";
export { default as ComboboxSeparator } from "../select/select-separator.vue";
export { default as ComboboxViewport } from "../select/select-viewport.vue";
export { default as ComboboxVirtualizer } from "../select/select-virtualizer.vue";
export {
  containsComboboxFilter,
  inlineComboboxCompletion,
  normalizeComboboxText,
  startsWithComboboxFilter,
} from "./combobox-filter.ts";
export type {
  ComboboxAriaInvalid,
  ComboboxAutocomplete,
  ComboboxChipSlotState,
  ComboboxFilter,
  ComboboxLoadContext,
  ComboboxLoader,
  ComboboxLoadStatus,
  ComboboxRootExpose,
  ComboboxRootProps,
  ComboboxSelectionMode,
  ComboboxSlotState,
  ComboboxState,
} from "./combobox-types.ts";
export type {
  SelectBy as ComboboxBy,
  SelectItemSlotState as ComboboxItemSlotState,
  SelectModelValue as ComboboxModelValue,
} from "../select/select-types.ts";
