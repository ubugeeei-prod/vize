import { h, nextTick } from "vue";

import TreeItemCheckbox from "./tree-item-checkbox.vue";
import TreeItemToggle from "./tree-item-toggle.vue";
import TreeItem from "./tree-item.vue";
import type { TreeFlatNode, TreeItemSlotState, TreeSlotState } from "./tree-types.ts";

/** File-system shaped fixture node. */
export interface FileNode {
  readonly id: string;
  readonly name: string;
  readonly children?: readonly FileNode[];
  readonly disabled?: boolean;
}

export const files: readonly FileNode[] = [
  {
    id: "src",
    name: "src",
    children: [
      { id: "app", name: "App.vue" },
      { id: "lib", name: "lib", children: [{ id: "util", name: "util.ts" }] },
    ],
  },
  { id: "docs", name: "docs", children: [{ id: "guide", name: "guide.md" }] },
  { id: "readme", name: "README.md" },
];

export const fileProps = {
  ariaLabel: "Files",
  getChildren: (node: FileNode) => node.children,
  getKey: (node: FileNode) => node.id,
  isDisabled: (node: FileNode) => node.disabled === true,
  items: files,
};

/** Event names recorded by file tree interaction tests. */
export const recordedTreeEvents = [
  "update:expanded",
  "update:selected",
  "update:checked",
  "action",
  "load",
  "loadError",
] as const;

/** Render every slot row as a TreeItem with a toggle, checkbox, and label. */
export function renderRows({ items }: TreeSlotState<FileNode, string>) {
  return items.map((item: TreeFlatNode<FileNode, string>) =>
    h(TreeItem, { key: item.key, item }, (slot: TreeItemSlotState<FileNode, string>) => [
      h(
        TreeItemToggle,
        {},
        {
          default: ({ state }: { readonly state: string }) =>
            h("i", { "data-toggle-state": state }),
        },
      ),
      h(
        TreeItemCheckbox,
        {},
        {
          default: ({ checked }: { readonly checked: string }) =>
            h("i", { "data-check-state": checked }),
        },
      ),
      h("span", { "data-name": slot.node.name }, slot.node.name),
    ]),
  );
}

/** Find a rendered row by key. */
export function row(handle: { root(): HTMLElement }, key: string): HTMLElement {
  const element = handle
    .root()
    .querySelector<HTMLElement>(`[data-vize-ui="tree-item"][id$="-item-key-${key}"]`);
  if (element === null) throw new Error(`Row ${key} is not rendered`);
  return element;
}

/** Keys of every rendered row, in DOM order. */
export function renderedKeys(handle: { root(): HTMLElement }): string[] {
  return [...handle.root().querySelectorAll<HTMLElement>('[data-vize-ui="tree-item"]')].map(
    (element) => element.querySelector("[data-name]")?.getAttribute("data-name") ?? "",
  );
}

/** Dispatch a keydown with modifiers that the shared harness does not model. */
export async function keydown(
  target: Element,
  key: string,
  init: KeyboardEventInit = {},
): Promise<KeyboardEvent> {
  const event = new KeyboardEvent("keydown", { key, bubbles: true, cancelable: true, ...init });
  target.dispatchEvent(event);
  await nextTick();
  return event;
}

/** Click with modifier keys. */
export async function clickWith(target: Element, init: MouseEventInit = {}): Promise<void> {
  target.dispatchEvent(new MouseEvent("click", { bubbles: true, cancelable: true, ...init }));
  await nextTick();
}

/** Wait for queued promise callbacks and a render flush. */
export async function settle(): Promise<void> {
  await new Promise((resolve) => setTimeout(resolve, 0));
  await nextTick();
}
