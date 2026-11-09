/** Compile-only contracts for generic collection component values and events. */

import Select from "./FrescoSelect.vue";

interface CollectionUser {
  readonly id: number;
  readonly name: string;
}

Select<CollectionUser>({
  options: [{ label: "Ada", value: { id: 1, name: "Ada" } }],
  modelValue: { id: 1, name: "Copy" },
  by: "id",
  "onUpdate:modelValue": (value) => value.name,
  onSelect: (value) => value.id,
});

Select<number>({
  options: [{ label: "One", value: 1 }],
  modelValue: 1,
  "onUpdate:modelValue": (value) => value.toFixed(),
});

Select<CollectionUser>({
  options: [],
  // @ts-expect-error - the model must match the generic option value.
  modelValue: "ada",
});

Select<CollectionUser>({
  options: [],
  // @ts-expect-error - comparison keys must exist on the value type.
  by: "email",
});
