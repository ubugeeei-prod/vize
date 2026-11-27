export type {
  InputGroupButtonProps,
  InputGroupInputProps,
  InputGroupProps,
  InputGroupTextareaProps,
} from "./types.ts";

/** Group that disables its fields and buttons together. */
export { default as InputGroup } from "./input-group.vue";

/** Text or icon addon inside an input group. */
export { default as InputGroupAddon } from "./input-group-addon.vue";

/** Text field that inherits the group's disabled state. */
export { default as InputGroupInput } from "./input-group-input.vue";

/** Textarea that inherits the group's disabled state. */
export { default as InputGroupTextarea } from "./input-group-textarea.vue";

/** Button that inherits the group's disabled state. */
export { default as InputGroupButton } from "./input-group-button.vue";
