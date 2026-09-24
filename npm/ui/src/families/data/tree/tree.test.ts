import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { h, nextTick } from "vue";

import type { TreeRootExpose } from "./tree.ts";
import TreeItemCheckbox from "./tree-item-checkbox.vue";
import TreeItemToggle from "./tree-item-toggle.vue";
import TreeItem from "./tree-item.vue";
import TreeRoot from "./tree-root.vue";
import {
  clickWith,
  keydown,
  fileProps,
  recordedTreeEvents,
  renderRows,
  renderedKeys,
  row,
  settle,
} from "./tree-test-utils.ts";
import type { TreeFlatNode, TreeSlotState } from "./tree-types.ts";
import { mountInteraction } from "../../../testing/mount.ts";

function mountFileTree(props: Record<string, unknown> = {}) {
  return mountInteraction(TreeRoot, {
    props: { ...fileProps, ...props },
    record: recordedTreeEvents,
    slots: { default: renderRows },
  });
}

test("renders APG tree semantics with flat levels, set positions, and roving tabindex", () => {
  const handle = mountFileTree({ id: "files", defaultExpanded: ["src"] });
  const tree = handle.getByRole("tree", { name: "Files" });
  const src = row(handle, "src");
  const app = row(handle, "app");
  const lib = row(handle, "lib");
  const readme = row(handle, "readme");

  assert.equal(tree.id, "files");
  assert.equal(tree.getAttribute("data-vize-ui"), "tree");
  assert.equal(tree.getAttribute("data-state"), "ready");
  assert.equal(tree.getAttribute("aria-multiselectable"), null);
  assert.deepEqual(renderedKeys(handle), ["src", "App.vue", "lib", "docs", "README.md"]);
  assert.equal(src.id, "files-item-key-src");
  assert.equal(src.getAttribute("role"), "treeitem");
  assert.equal(src.getAttribute("aria-level"), "1");
  assert.equal(src.getAttribute("aria-posinset"), "1");
  assert.equal(src.getAttribute("aria-setsize"), "3");
  assert.equal(src.getAttribute("aria-expanded"), "true");
  assert.equal(src.getAttribute("aria-selected"), "false");
  assert.equal(src.getAttribute("data-state"), "expanded");
  assert.equal(src.getAttribute("tabindex"), "0");
  assert.equal(app.getAttribute("aria-level"), "2");
  assert.equal(app.getAttribute("aria-setsize"), "2");
  assert.equal(app.getAttribute("aria-expanded"), null, "leaves omit aria-expanded");
  assert.equal(app.getAttribute("data-state"), "leaf");
  assert.equal(app.getAttribute("tabindex"), "-1");
  assert.equal(lib.getAttribute("aria-expanded"), "false");
  assert.equal(lib.getAttribute("data-state"), "collapsed");
  assert.equal(readme.getAttribute("aria-posinset"), "3");
  assert.equal(
    src.querySelector("[data-toggle-state]")?.getAttribute("data-toggle-state"),
    "expanded",
  );
  handle.unmount();
});

test("arrow keys, Home, and End follow the APG tree keyboard model", async () => {
  const handle = mountFileTree();
  const src = row(handle, "src");

  assert.ok((await handle.tab()) === src);
  await handle.press(src, "ArrowRight");
  assert.equal(src.getAttribute("aria-expanded"), "true", "Right expands a closed parent");
  assert.ok(handle.activeElement() === src);
  await handle.press(src, "ArrowRight");
  assert.ok(handle.activeElement() === row(handle, "app"), "Right on an open parent enters it");
  await handle.press(row(handle, "app"), "ArrowDown");
  assert.ok(handle.activeElement() === row(handle, "lib"));
  await handle.press(row(handle, "lib"), "ArrowLeft");
  assert.ok(handle.activeElement() === src, "Left on a closed child moves to its parent");
  await handle.press(src, "ArrowLeft");
  assert.equal(src.getAttribute("aria-expanded"), "false", "Left collapses an open parent");
  const end = await handle.press(src, "End");
  assert.equal(end.keydownPrevented, true);
  assert.ok(handle.activeElement() === row(handle, "readme"));
  assert.equal(row(handle, "readme").getAttribute("tabindex"), "0");
  assert.equal(src.getAttribute("tabindex"), "-1");
  await handle.press(row(handle, "readme"), "Home");
  assert.ok(handle.activeElement() === src);
  await handle.press(src, "ArrowUp");
  assert.ok(handle.activeElement() === src, "ArrowUp stops at the first row");
  assert.deepEqual(
    handle.wrapper.emitted("update:expanded")?.map(([keys]) => keys),
    [["src"], []],
  );
  handle.unmount();
});

test("collapsing an ancestor moves focus from a hidden descendant to the ancestor", async () => {
  const handle = mountFileTree({ defaultExpanded: ["src", "lib"] });
  const util = row(handle, "util");
  util.focus();
  await nextTick();
  await handle.press(util, "ArrowLeft");
  assert.ok(handle.activeElement() === row(handle, "lib"));

  const tree = handle.exposes<TreeRootExpose<string>>();
  assert.equal(tree.collapse("src"), true);
  await nextTick();
  assert.ok(handle.activeElement() === row(handle, "src"));
  assert.equal(tree.activeKey, "src");
  assert.deepEqual(renderedKeys(handle), ["src", "docs", "README.md"]);
  handle.unmount();
});

test("RTL maps ArrowLeft to expand and ArrowRight to collapse", async () => {
  const handle = mountFileTree({ dir: "rtl" });
  const src = row(handle, "src");
  src.focus();
  await handle.press(src, "ArrowLeft");
  assert.equal(src.getAttribute("aria-expanded"), "true");
  await handle.press(src, "ArrowRight");
  assert.equal(src.getAttribute("aria-expanded"), "false");
  assert.equal(handle.root().getAttribute("dir"), "rtl");
  handle.unmount();
});

test("single selection follows click and Enter, and emits action for activation", async () => {
  const handle = mountFileTree({ defaultExpanded: ["docs"] });
  const docs = row(handle, "docs");
  const guide = row(handle, "guide");

  await handle.click(guide);
  assert.equal(guide.getAttribute("aria-selected"), "true");
  assert.equal(guide.getAttribute("data-selected"), "true");
  assert.ok(handle.activeElement() === guide);
  await handle.press(guide, "ArrowUp");
  assert.equal(guide.getAttribute("aria-selected"), "true", "focus alone does not select");
  await handle.press(docs, "Enter");
  assert.equal(docs.getAttribute("aria-selected"), "true");
  assert.equal(guide.getAttribute("aria-selected"), "false");
  assert.deepEqual(
    handle.wrapper.emitted("update:selected")?.map(([keys]) => keys),
    [["guide"], ["docs"]],
  );
  const action = handle.wrapper.emitted("action")?.[0];
  assert.equal(action?.[0], "docs");
  assert.ok(action?.[2] instanceof KeyboardEvent);
  handle.unmount();

  const follow = mountFileTree({ selectionFollowsFocus: true });
  const src = row(follow, "src");
  src.focus();
  await follow.press(src, "ArrowDown");
  assert.equal(row(follow, "docs").getAttribute("aria-selected"), "true");
  follow.unmount();
});

test("multiple selection toggles, extends ranges, and selects all", async () => {
  const handle = mountFileTree({ selectionMode: "multiple", defaultExpanded: ["src"] });
  const tree = handle.root();
  const src = row(handle, "src");
  const lib = row(handle, "lib");
  const docs = row(handle, "docs");

  assert.equal(tree.getAttribute("aria-multiselectable"), "true");
  await handle.click(src);
  await handle.click(lib);
  assert.equal(src.getAttribute("aria-selected"), "true");
  assert.equal(lib.getAttribute("aria-selected"), "true");
  await handle.click(src);
  assert.equal(src.getAttribute("aria-selected"), "false", "click toggles in multiple mode");

  await clickWith(docs, { shiftKey: true });
  assert.deepEqual(handle.exposes<TreeRootExpose<string>>().selected, [
    "src",
    "app",
    "lib",
    "docs",
  ]);

  await handle.press(docs, " ");
  assert.equal(docs.getAttribute("aria-selected"), "false", "Space toggles the focused row");
  await keydown(docs, "ArrowDown", { shiftKey: true });
  assert.ok(handle.activeElement() === row(handle, "readme"));
  assert.equal(row(handle, "readme").getAttribute("aria-selected"), "true");

  await keydown(row(handle, "readme"), "a", { ctrlKey: true });
  assert.deepEqual(handle.exposes<TreeRootExpose<string>>().selected, [
    "src",
    "app",
    "lib",
    "docs",
    "readme",
  ]);
  await keydown(row(handle, "readme"), "a", { metaKey: true });
  assert.deepEqual(handle.exposes<TreeRootExpose<string>>().selected, []);
  handle.unmount();
});

test("selection mode none omits aria-selected and ignores selection input", async () => {
  const handle = mountFileTree({ selectionMode: "none" });
  const src = row(handle, "src");
  await handle.click(src);
  await handle.press(src, "Enter");
  assert.equal(src.getAttribute("aria-selected"), null);
  assert.equal(handle.wrapper.emitted("update:selected"), undefined);
  assert.equal(handle.wrapper.emitted("action")?.length, 1);
  handle.unmount();
});

test("checkboxes cascade to descendants and derive tri-state ancestors", async () => {
  const handle = mountFileTree({ checkable: true, defaultExpanded: ["src", "lib"] });
  const src = row(handle, "src");
  const app = row(handle, "app");
  const util = row(handle, "util");

  assert.equal(handle.root().getAttribute("data-checkable"), "true");
  assert.equal(src.getAttribute("aria-checked"), "false");
  src.focus();
  await handle.press(src, " ");
  assert.equal(src.getAttribute("aria-checked"), "true");
  assert.equal(app.getAttribute("aria-checked"), "true");
  assert.equal(util.getAttribute("aria-checked"), "true");
  assert.equal(src.getAttribute("aria-selected"), "false", "Space checks instead of selecting");
  assert.deepEqual(handle.wrapper.emitted("update:checked")?.at(-1)?.[0], [
    "src",
    "app",
    "lib",
    "util",
  ]);

  const utilCheckbox = util.querySelector('[data-vize-ui="tree-item-checkbox"]');
  assert.ok(utilCheckbox instanceof HTMLElement);
  await handle.click(utilCheckbox);
  assert.equal(util.getAttribute("aria-checked"), "false");
  assert.equal(row(handle, "lib").getAttribute("aria-checked"), "false");
  assert.equal(src.getAttribute("aria-checked"), "mixed");
  assert.equal(src.getAttribute("data-checked"), "mixed");
  assert.equal(src.querySelector("[data-check-state]")?.getAttribute("data-check-state"), "mixed");
  assert.deepEqual(handle.wrapper.emitted("update:checked")?.at(-1)?.[0], ["app"]);
  assert.equal(util.getAttribute("aria-selected"), "false", "checkbox clicks do not select");

  await handle.press(src, " ");
  assert.equal(src.getAttribute("aria-checked"), "true", "mixed parents check their subtree");
  handle.unmount();
});

test("independent checkboxes toggle only the requested node", async () => {
  const handle = mountFileTree({
    checkable: true,
    checkPropagation: "independent",
    defaultExpanded: ["src"],
  });
  const src = row(handle, "src");
  src.focus();
  await handle.press(src, " ");
  assert.equal(src.getAttribute("aria-checked"), "true");
  assert.equal(row(handle, "app").getAttribute("aria-checked"), "false");
  handle.unmount();
});

test("lazy children load on expand with busy state, load events, and inherited checks", async () => {
  interface LazyNode {
    readonly id: string;
    readonly children?: readonly LazyNode[];
  }
  let resolveLoad: (children: readonly LazyNode[]) => void = () => {};
  const requested: string[] = [];
  const handle = mountInteraction(TreeRoot, {
    props: {
      ariaLabel: "Remote",
      checkable: true,
      defaultChecked: ["remote"],
      getKey: (node: LazyNode) => node.id,
      getChildren: (node: LazyNode) => node.children,
      hasChildren: (node: LazyNode) => node.id !== "leaf",
      items: [{ id: "remote" }, { id: "leaf" }],
      loadChildren: (node: LazyNode) => {
        requested.push(node.id);
        return new Promise<readonly LazyNode[]>((resolve) => {
          resolveLoad = resolve;
        });
      },
    },
    record: ["load", "update:expanded"],
    slots: { default: renderLazyRows },
  });
  const remote = handle.root().querySelector<HTMLElement>('[data-key="remote"]');
  const leaf = handle.root().querySelector<HTMLElement>('[data-key="leaf"]');
  assert.ok(remote && leaf);
  assert.equal(remote.getAttribute("aria-expanded"), "false", "unloaded branches are expandable");
  assert.equal(leaf.getAttribute("aria-expanded"), null, "hasChildren false marks a leaf");

  remote.focus();
  await handle.press(remote, "ArrowRight");
  assert.deepEqual(requested, ["remote"]);
  assert.equal(remote.getAttribute("aria-expanded"), "true");
  assert.equal(remote.getAttribute("aria-busy"), "true");
  assert.equal(remote.getAttribute("data-load-state"), "loading");

  resolveLoad([{ id: "child-a" }, { id: "child-b" }]);
  await settle();
  assert.equal(remote.getAttribute("aria-busy"), null);
  assert.equal(remote.getAttribute("data-load-state"), "loaded");
  const childA = handle.root().querySelector<HTMLElement>('[data-key="child-a"]');
  assert.equal(childA?.getAttribute("aria-level"), "2");
  assert.equal(childA?.getAttribute("aria-checked"), "true", "children inherit a checked parent");
  assert.deepEqual(handle.wrapper.emitted("load")?.[0]?.[0], "remote");

  await handle.press(remote, "ArrowLeft");
  await handle.press(remote, "ArrowRight");
  assert.deepEqual(requested, ["remote"], "loaded children are cached");
  handle.unmount();
});

test("failed lazy loads expose an error state and can reload", async () => {
  interface LazyNode {
    readonly id: string;
  }
  let fail = true;
  const handle = mountInteraction(TreeRoot, {
    props: {
      ariaLabel: "Remote",
      getKey: (node: LazyNode) => node.id,
      items: [{ id: "remote" }],
      loadChildren: () =>
        fail ? Promise.reject(new Error("offline")) : Promise.resolve([{ id: "late" }]),
    },
    record: ["loadError", "load"],
    slots: { default: renderLazyRows },
  });
  const tree = handle.exposes<TreeRootExpose<string>>();
  await tree.expand("remote");
  await settle();
  const remote = handle.root().querySelector<HTMLElement>('[data-key="remote"]');
  assert.equal(remote?.getAttribute("data-load-state"), "error");
  const loadError = handle.wrapper.emitted("loadError")?.[0];
  assert.equal(loadError?.[0], "remote");
  assert.ok(loadError?.[1] instanceof Error);

  fail = false;
  assert.equal(await tree.reload("remote"), true);
  await settle();
  assert.equal(remote?.getAttribute("data-load-state"), "loaded");
  assert.ok(handle.root().querySelector('[data-key="late"]'));
  handle.unmount();
});

test("typeahead moves focus to the next row whose text matches", async () => {
  const handle = mountFileTree();
  const src = row(handle, "src");
  src.focus();
  await keydown(src, "d");
  assert.ok(handle.activeElement() === row(handle, "docs"));
  await settle();
  await new Promise((resolve) => setTimeout(resolve, 520));
  await keydown(row(handle, "docs"), "R");
  assert.ok(handle.activeElement() === row(handle, "readme"));
  handle.unmount();
});

test("asterisk expands siblings and expandAll/collapseAll cover every resolved branch", async () => {
  const handle = mountFileTree();
  const src = row(handle, "src");
  src.focus();
  await keydown(src, "*");
  assert.equal(src.getAttribute("aria-expanded"), "true");
  assert.equal(row(handle, "docs").getAttribute("aria-expanded"), "true");
  assert.equal(row(handle, "lib").getAttribute("aria-expanded"), "false");

  const tree = handle.exposes<TreeRootExpose<string>>();
  assert.equal(tree.expandAll(), true);
  await nextTick();
  assert.deepEqual(renderedKeys(handle), [
    "src",
    "App.vue",
    "lib",
    "util.ts",
    "docs",
    "guide.md",
    "README.md",
  ]);
  assert.equal(tree.collapseAll(), true);
  await nextTick();
  assert.deepEqual(renderedKeys(handle), ["src", "docs", "README.md"]);
  handle.unmount();
});

test("disabled rows stay focusable but cannot expand, select, or check", async () => {
  const handle = mountFileTree({
    checkable: true,
    isDisabled: (node: { readonly id: string }) => node.id === "src",
  });
  const src = row(handle, "src");
  assert.equal(src.getAttribute("aria-disabled"), "true");
  await handle.click(src);
  src.focus();
  await handle.press(src, "ArrowRight");
  await handle.press(src, " ");
  await handle.press(src, "Enter");
  assert.equal(src.getAttribute("aria-expanded"), "false");
  assert.equal(src.getAttribute("aria-selected"), "false");
  assert.equal(src.getAttribute("aria-checked"), "false");
  assert.equal(handle.wrapper.emitted("action"), undefined);
  await handle.press(src, "ArrowDown");
  assert.ok(handle.activeElement() === row(handle, "docs"), "disabled rows do not trap focus");
  handle.unmount();

  const disabledTree = mountFileTree({ disabled: true });
  assert.equal(disabledTree.root().getAttribute("aria-disabled"), "true");
  assert.equal(disabledTree.root().getAttribute("data-state"), "disabled");
  assert.equal(row(disabledTree, "src").getAttribute("tabindex"), "-1");
  assert.equal(await disabledTree.tab(), null);
  disabledTree.unmount();
});

test("controlled expansion and selection wait for the parent to accept requests", async () => {
  const handle = mountFileTree({ expanded: [], selected: [] });
  const src = row(handle, "src");
  await handle.click(src);
  src.focus();
  await handle.press(src, "ArrowRight");
  assert.equal(src.getAttribute("aria-expanded"), "false");
  assert.equal(src.getAttribute("aria-selected"), "false");
  assert.deepEqual(handle.wrapper.emitted("update:expanded")?.[0]?.[0], ["src"]);
  assert.deepEqual(handle.wrapper.emitted("update:selected")?.[0]?.[0], ["src"]);

  await handle.wrapper.setProps({ expanded: ["src"], selected: ["src"] });
  assert.equal(src.getAttribute("aria-expanded"), "true");
  assert.equal(src.getAttribute("aria-selected"), "true");
  handle.unmount();
});

test("the toggle expands on click without selecting, and expandOnClick toggles rows", async () => {
  const handle = mountFileTree();
  const src = row(handle, "src");
  const toggle = src.querySelector('[data-vize-ui="tree-item-toggle"]');
  assert.ok(toggle instanceof HTMLElement);
  assert.equal(toggle.getAttribute("aria-hidden"), "true");
  await handle.click(toggle);
  assert.equal(src.getAttribute("aria-expanded"), "true");
  assert.equal(src.getAttribute("aria-selected"), "false");
  assert.ok(handle.activeElement() === src);
  handle.unmount();

  const rows = mountFileTree({ expandOnClick: true });
  await rows.click(row(rows, "docs"));
  assert.equal(row(rows, "docs").getAttribute("aria-expanded"), "true");
  assert.equal(row(rows, "docs").getAttribute("aria-selected"), "true");
  await clickWith(row(rows, "readme"), { detail: 2 });
  assert.equal(rows.wrapper.emitted("action")?.[0]?.[0], "readme", "double click activates");
  rows.unmount();
});

test("exposes typed state and imperative focus, selection, and checkbox controls", async () => {
  const handle = mountFileTree({ checkable: true, selectionMode: "multiple" });
  const tree = handle.exposes<TreeRootExpose<string>>();
  tree.focus();
  assert.ok(handle.activeElement() === row(handle, "src"));
  assert.equal(await tree.expand("docs"), true);
  await nextTick();
  assert.equal(tree.focusKey("guide"), true);
  assert.ok(handle.activeElement() === row(handle, "guide"));
  assert.equal(tree.focusKey("util"), false, "hidden rows cannot take focus");
  assert.equal(tree.setSelected(["docs", "guide"]), true);
  assert.deepEqual(tree.selected, ["docs", "guide"]);
  assert.equal(tree.toggleChecked("docs"), true);
  assert.equal(tree.getCheckedState("guide"), "checked");
  assert.equal(tree.getCheckedState("src"), "unchecked");
  assert.equal(await tree.toggle("docs"), true);
  assert.deepEqual(tree.expanded, []);
  assert.match(tree.id, /tree$/);
  assert.ok(tree.element instanceof HTMLDivElement);
  handle.unmount();
});

test("compound parts require a matching root provider", () => {
  assert.throws(
    () => mountInteraction(TreeItem, { props: { item: { key: "a" } } }),
    /VIZE_UI_CONTEXT_MISSING/,
  );
  assert.throws(() => mountInteraction(TreeItemToggle), /VIZE_UI_CONTEXT_MISSING/);
  assert.throws(() => mountInteraction(TreeItemCheckbox), /VIZE_UI_CONTEXT_MISSING/);
});

function renderLazyRows({ items }: TreeSlotState<{ readonly id: string }, string>) {
  return items.map((item: TreeFlatNode<{ readonly id: string }, string>) =>
    h(TreeItem, { key: item.key, item, "data-key": item.key }, () => item.key),
  );
}
