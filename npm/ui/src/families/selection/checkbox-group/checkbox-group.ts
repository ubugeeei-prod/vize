/** Typed multi-select group of native checkboxes with an optional tri-state select-all parent. */
export { default as CheckboxGroup } from "./checkbox-group.vue";
/** One native checkbox bound to an option of the nearest CheckboxGroup. */
export { default as CheckboxGroupItem } from "./checkbox-group-item.vue";
/** Tri-state parent checkbox that selects or clears every enabled option. */
export { default as CheckboxGroupSelectAll } from "./checkbox-group-select-all.vue";
export type {
  CheckboxGroupAriaInvalid,
  CheckboxGroupExpose,
  CheckboxGroupItemState,
  CheckboxGroupKey,
  CheckboxGroupOrientation,
  CheckboxGroupSlotState,
  CheckboxGroupState,
} from "./checkbox-group-types.ts";
