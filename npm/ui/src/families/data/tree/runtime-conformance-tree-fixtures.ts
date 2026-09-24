import assert from "node:assert/strict";

import { h } from "vue";

import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";
import { TreeItem, TreeItemCheckbox, TreeItemToggle, TreeRoot } from "./tree.ts";
import type { TreeFlatNode, TreeItemSlotState, TreeSlotState } from "./tree-types.ts";

interface FixtureNode {
  readonly id: string;
  readonly label: string;
  readonly children?: readonly FixtureNode[];
}

// Instantiation expressions pin the generic SFCs so `h()` checks props against real node types.
const FixtureTreeRoot = TreeRoot<FixtureNode, string>;
const FixtureTreeItem = TreeItem<FixtureNode, string>;

const nodes: readonly FixtureNode[] = [
  {
    id: "docs",
    label: "Docs",
    children: [
      { id: "intro", label: "Intro" },
      { id: "api", label: "API" },
    ],
  },
  { id: "blog", label: "Blog" },
];

function renderTree(
  row: (slot: TreeItemSlotState<FixtureNode, string>) => unknown,
  defaultSelected: readonly string[] = [],
) {
  return h(
    FixtureTreeRoot,
    {
      ariaLabel: "Site",
      checkable: true,
      defaultChecked: ["intro"],
      defaultExpanded: ["docs"],
      getChildren: (node: FixtureNode) => node.children,
      getKey: (node: FixtureNode) => node.id,
      id: "site-tree",
      items: nodes,
      defaultSelected,
    },
    {
      default: ({ items }: TreeSlotState<FixtureNode, string>) =>
        items.map((item: TreeFlatNode<FixtureNode, string>) =>
          h(FixtureTreeItem, { key: item.key, item }, { default: row }),
        ),
    },
  );
}

export const treeRuntimeFixtures: readonly RuntimeFixture[] = [
  {
    name: "tree",
    sourceFile: "families/data/tree/tree-root.vue",
    render: () => renderTree(({ node }) => node.label),
    assertServerMarkup(html) {
      assert.match(html, /id="site-tree"/);
      assert.match(html, /role="tree"/);
      assert.match(html, /data-vize-ui="tree"/);
      assert.match(html, /aria-label="Site"/);
      assert.match(html, /role="treeitem"/);
      assert.match(html, /id="site-tree-item-key-docs"/);
      assert.match(html, /aria-expanded="true"/);
      assert.match(html, /aria-checked="mixed"/);
      assert.match(html, /Intro/);
    },
    assertHydratedDom(host) {
      const tree = host.querySelector('[data-vize-ui="tree"]');
      const docs = host.querySelector("#site-tree-item-key-docs");
      assert.ok(tree instanceof HTMLDivElement);
      assert.equal(tree.getAttribute("role"), "tree");
      assert.ok(docs instanceof HTMLDivElement);
      assert.equal(docs.getAttribute("aria-expanded"), "true");
      assert.equal(docs.getAttribute("aria-checked"), "mixed");
      assert.equal(docs.getAttribute("tabindex"), "0");
      assert.equal(host.querySelectorAll('[role="treeitem"]').length, 4);
    },
  },
  {
    name: "tree-item",
    sourceFile: "families/data/tree/tree-item.vue",
    render: () => renderTree(({ level, state }) => `${level}:${state}`, ["blog"]),
    assertServerMarkup(html) {
      assert.match(html, /aria-level="2"/);
      assert.match(html, /aria-setsize="2"/);
      assert.match(html, /aria-posinset="2"/);
      assert.match(html, /aria-selected="true"/);
      assert.match(html, /1:expanded/);
      assert.match(html, /2:leaf/);
    },
    assertHydratedDom(host) {
      const blog = host.querySelector("#site-tree-item-key-blog");
      assert.ok(blog instanceof HTMLDivElement);
      assert.equal(blog.getAttribute("aria-selected"), "true");
      assert.equal(blog.getAttribute("tabindex"), "0", "the first selected row owns roving focus");
      assert.equal(blog.textContent, "1:leaf");
    },
  },
  {
    name: "tree-item-toggle",
    sourceFile: "families/data/tree/tree-item-toggle.vue",
    render: () =>
      renderTree(() =>
        h(TreeItemToggle, null, {
          default: ({ state }: { readonly state: string }) => state,
        }),
      ),
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="tree-item-toggle"/);
      assert.match(html, /aria-hidden="true"/);
      assert.match(html, /expanded/);
      assert.match(html, /leaf/);
    },
    assertHydratedDom(host) {
      const toggle = host.querySelector('[data-vize-ui="tree-item-toggle"]');
      assert.ok(toggle instanceof HTMLSpanElement);
      assert.equal(toggle.getAttribute("data-state"), "expanded");
      assert.equal(toggle.getAttribute("role"), "presentation");
    },
  },
  {
    name: "tree-item-checkbox",
    sourceFile: "families/data/tree/tree-item-checkbox.vue",
    render: () =>
      renderTree(() =>
        h(TreeItemCheckbox, null, {
          default: ({ checked }: { readonly checked: string }) => checked,
        }),
      ),
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="tree-item-checkbox"/);
      assert.match(html, /data-state="mixed"/);
      assert.match(html, /data-state="checked"/);
    },
    assertHydratedDom(host) {
      const checkbox = host.querySelector('[data-vize-ui="tree-item-checkbox"]');
      assert.ok(checkbox instanceof HTMLSpanElement);
      assert.equal(checkbox.getAttribute("data-state"), "mixed");
      assert.equal(checkbox.getAttribute("aria-hidden"), "true");
    },
  },
];
