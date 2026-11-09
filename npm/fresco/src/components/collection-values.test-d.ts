/** Compile-only contracts for generic collection component values and events. */

import List from "./FrescoList.vue";
import Menu from "./FrescoMenu.vue";
import Select from "./FrescoSelect.vue";
import Tree from "./FrescoTree.vue";
import type { ListProps } from "./List.js";
import type { MenuProps } from "./Menu.js";
import type { TreeProps } from "./Tree.js";

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

export const listAcceptsLegacyStringKeys: ListProps = {
  items: [{ key: "alpha", label: "Alpha" }],
  modelValue: "alpha",
};

List<CollectionUser>({
  items: [{ key: "ada", label: "Ada", value: { id: 1, name: "Ada" } }],
  modelValue: { id: 1, name: "Copy" },
  by: "id",
  "onUpdate:modelValue": (value) => value.name,
  onSelect: (value) => value.id,
});

List<CollectionUser>({
  // @ts-expect-error - non-string list values require an explicit value.
  items: [{ key: "ada", label: "Ada" }],
});

List<CollectionUser>({
  items: [],
  // @ts-expect-error - the model must match the generic item value.
  modelValue: "ada",
});

export const menuAcceptsBoundedViewport: MenuProps = {
  items: [{ key: "open", label: "Open" }],
  maxHeight: 5,
};

Menu<CollectionUser>({
  items: [{ key: "ada", label: "Ada", value: { id: 1, name: "Ada" } }],
  maxHeight: 5,
  onSelect: (value) => value.name,
  "onUpdate:focusedIndex": (index) => index.toFixed(),
});

Menu<CollectionUser>({
  // @ts-expect-error - non-string menu values require an explicit value.
  items: [{ key: "ada", label: "Ada" }],
});

Menu<CollectionUser>({
  items: [],
  // @ts-expect-error - viewport sizes are numeric item counts.
  maxHeight: "5",
});

export const treeAcceptsLegacyStringKeys: TreeProps = {
  data: [{ key: "root", label: "Root" }],
  selected: "root",
  expanded: ["root"],
  maxHeight: 6,
};

Tree<CollectionUser>({
  data: [
    {
      key: "root",
      label: "Root",
      value: { id: 1, name: "Root" },
      children: [{ key: "child", label: "Child", value: { id: 2, name: "Child" } }],
    },
  ],
  selected: { id: 2, name: "Copy" },
  by: "id",
  maxHeight: 6,
  "onUpdate:selected": (value) => value.name,
  onSelect: (value) => value.id,
  onToggle: (key, expanded) => `${key}:${expanded}`,
  "onUpdate:expanded": (keys) => keys.join(","),
});

Tree<CollectionUser>({
  // @ts-expect-error - non-string tree values require an explicit value.
  data: [{ key: "root", label: "Root" }],
});

Tree<CollectionUser>({
  data: [],
  // @ts-expect-error - comparison keys must exist on the tree value type.
  by: "email",
});
