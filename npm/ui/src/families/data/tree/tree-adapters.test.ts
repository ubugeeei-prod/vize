import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick } from "vue";

import { useTreeReorder, useTreeVirtualizer } from "./tree.ts";
import type {
  TreeFlatNode,
  TreeMoveEvent,
  TreeReorderController,
  TreeRootExpose,
  TreeSlotState,
  TreeVirtualizer,
} from "./tree.ts";
import TreeItem from "./tree-item.vue";
import TreeRoot from "./tree-root.vue";
import { fileProps, keydown, renderRows, row } from "./tree-test-utils.ts";
import { mountInteraction } from "../../../testing/mount.ts";

interface NumberNode {
  readonly id: number;
  readonly label: string;
}

const manyNodes: readonly NumberNode[] = Array.from({ length: 200 }, (_, index) => ({
  id: index,
  label: `Row ${index}`,
}));

function mountVirtualTree() {
  const refs: {
    virtualizer: TreeVirtualizer | null;
    tree: TreeRootExpose<number> | null;
  } = { virtualizer: null, tree: null };
  const Probe = defineComponent({
    name: "TreeVirtualProbe",
    setup() {
      const controller = useTreeVirtualizer({
        itemSize: 20,
        overscan: 1,
        initialRect: { width: 200, height: 60 },
      });
      refs.virtualizer = controller;
      return () =>
        h(
          TreeRoot,
          {
            ariaLabel: "Rows",
            getKey: (node: NumberNode) => node.id,
            getTextValue: (node: NumberNode) => node.label,
            items: manyNodes,
            ref: (value: unknown) => {
              refs.tree = value as TreeRootExpose<number> | null;
            },
            virtualizer: controller,
          },
          ({ items }: TreeSlotState<NumberNode, number>) =>
            h("div", { "data-spacer": String(controller.totalSize.value) }, [
              items.map((item: TreeFlatNode<NumberNode, number>) =>
                h(
                  TreeItem,
                  {
                    key: item.key,
                    item,
                    "data-start": String(controller.getItemStart(item.index)),
                  },
                  () => item.node.label,
                ),
              ),
            ]),
        );
    },
  });
  const handle = mountInteraction(Probe);
  const { tree, virtualizer } = refs;
  if (virtualizer === null || tree === null) assert.fail("virtual tree must mount");
  return { handle, tree, virtualizer };
}

function renderedRows(root: HTMLElement): HTMLElement[] {
  return [...root.querySelectorAll<HTMLElement>('[data-vize-ui="tree-item"]')];
}

test("a virtualizer renders only the window and keeps set metadata for every row", () => {
  const { handle, virtualizer } = mountVirtualTree();
  const rows = renderedRows(handle.root());

  assert.equal(handle.root().getAttribute("data-virtualized"), "true");
  assert.equal(rows.length, 4, "three visible rows plus one overscan row");
  assert.equal(rows[0]?.getAttribute("aria-setsize"), "200");
  assert.equal(rows[3]?.getAttribute("aria-posinset"), "4");
  assert.equal(rows[2]?.getAttribute("data-start"), "40");
  assert.equal(virtualizer.totalSize.value, 4000);
  assert.equal(handle.root().querySelector("[data-spacer]")?.getAttribute("data-spacer"), "4000");
  handle.unmount();
});

test("keyboard focus scrolls rows outside the window into view before focusing them", async () => {
  const { handle, tree } = mountVirtualTree();
  const first = renderedRows(handle.root())[0];
  assert.ok(first);
  first.focus();
  await handle.press(first, "End");
  await nextTick();
  await nextTick();
  const last = handle.root().querySelector<HTMLElement>('[aria-posinset="200"]');
  assert.ok(last, "the last row renders after End");
  assert.ok(handle.activeElement() === last);
  assert.equal(tree.activeKey, 199);

  await keydown(last, "R");
  await nextTick();
  assert.equal(tree.activeKey, 0, "typeahead reaches unmounted rows through getTextValue");
  handle.unmount();
});

test("the tree element takes sequential focus while the active row is outside the window", async () => {
  const { handle, tree } = mountVirtualTree();
  const root = handle.root();
  assert.equal(root.getAttribute("tabindex"), null);
  tree.focusKey(150);
  await nextTick();
  await nextTick();
  const viewport = root;
  viewport.scrollTop = 0;
  viewport.dispatchEvent(new Event("scroll"));
  await nextTick();
  assert.equal(root.getAttribute("tabindex"), "0");
  root.focus();
  await nextTick();
  await nextTick();
  assert.equal(document.activeElement?.getAttribute("aria-posinset"), "151");
  handle.unmount();
});

function mountReorderTree(onMove: (event: TreeMoveEvent<string>) => void, canMove?: () => boolean) {
  const refs: { controller: TreeReorderController<string> | null } = { controller: null };
  const Probe = defineComponent({
    name: "TreeReorderProbe",
    setup() {
      const reorder = useTreeReorder<string>({
        onMove,
        ...(canMove === undefined ? {} : { canMove }),
      });
      refs.controller = reorder;
      return () => h(TreeRoot, { ...fileProps, defaultExpanded: ["src"], reorder }, renderRows);
    },
  });
  const handle = mountInteraction(Probe);
  const { controller } = refs;
  if (controller === null) assert.fail("reorder controller must be created");
  return { handle, controller };
}

test("Alt+Arrow keys request sibling, nesting, and outdent moves", async () => {
  const moves: TreeMoveEvent<string>[] = [];
  const { handle } = mountReorderTree((move) => moves.push(move));
  const lib = row(handle, "lib");
  const app = row(handle, "app");
  lib.focus();

  const up = await keydown(lib, "ArrowUp", { altKey: true });
  assert.equal(up.defaultPrevented, true);
  await keydown(app, "ArrowDown", { altKey: true });
  await keydown(lib, "ArrowRight", { altKey: true });
  await keydown(lib, "ArrowLeft", { altKey: true });
  const blocked = await keydown(app, "ArrowUp", { altKey: true });
  assert.equal(blocked.defaultPrevented, false, "no previous sibling means no move");

  assert.deepEqual(
    moves.map(({ key, targetKey, position, source }) => [key, targetKey, position, source]),
    [
      ["lib", "app", "before", "keyboard"],
      ["app", "lib", "after", "keyboard"],
      ["lib", "app", "inside", "keyboard"],
      ["lib", "src", "after", "keyboard"],
    ],
  );
  assert.ok(moves[0]?.originalEvent instanceof KeyboardEvent);
  assert.equal(row(handle, "lib").getAttribute("aria-expanded"), "false", "moves do not expand");
  handle.unmount();
});

test("canMove vetoes moves and rows register with the pointer sort engine", async () => {
  const moves: TreeMoveEvent<string>[] = [];
  const { handle, controller } = mountReorderTree(
    (move) => moves.push(move),
    () => false,
  );
  const lib = row(handle, "lib");
  lib.focus();
  await keydown(lib, "ArrowUp", { altKey: true });
  assert.deepEqual(moves, []);
  assert.equal(
    controller.move({
      key: "lib",
      targetKey: "lib",
      position: "inside",
      source: "pointer",
      originalEvent: null,
    }),
    false,
  );
  assert.equal(controller.isDragging.value, false);
  assert.equal(controller.dropTarget.value, null);
  assert.equal(controller.cancel(), false);
  assert.equal(lib.getAttribute("data-dragging"), null);
  handle.unmount();
});
