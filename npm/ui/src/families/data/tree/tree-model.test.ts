import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import {
  deriveTreeCheckedStates,
  flattenVisibleTree,
  indexTree,
  normalizeTreeChecked,
  toggleTreeChecked,
} from "./tree.ts";
import { getTreeKeyIdSegment } from "./tree-model.ts";
import { files, type FileNode } from "./tree-test-utils.ts";

const accessors = {
  getKey: (node: FileNode) => node.id,
  resolveChildren: (node: FileNode) => node.children ?? null,
};

test("indexes every resolved node in pre-order with levels and set positions", () => {
  const index = indexTree(files, accessors);
  assert.deepEqual(index.rootKeys, ["src", "docs", "readme"]);
  assert.deepEqual(index.order, ["src", "app", "lib", "util", "docs", "guide", "readme"]);
  assert.deepEqual(
    { ...index.entries.get("util"), node: undefined },
    {
      key: "util",
      node: undefined,
      parentKey: "lib",
      level: 3,
      posinset: 1,
      setsize: 1,
      childKeys: null,
    },
  );
  assert.deepEqual(index.entries.get("src")?.childKeys, ["app", "lib"]);
});

test("rejects duplicate keys with a stable diagnostic", () => {
  assert.throws(
    () =>
      indexTree(
        [
          { id: "a", name: "A" },
          { id: "a", name: "B" },
        ],
        accessors,
      ),
    /VIZE_UI_TREE_DUPLICATE_KEY: tree key a is not unique/,
  );
});

test("flattens only expanded branches with visible indexes", () => {
  const index = indexTree(files, accessors);
  const rows = flattenVisibleTree(
    index,
    new Set(["src", "lib"]),
    (entry) => (entry.childKeys?.length ?? 0) > 0,
  );
  assert.deepEqual(
    rows.map((row) => [row.key, row.index, row.level, row.expanded]),
    [
      ["src", 0, 1, true],
      ["app", 1, 2, false],
      ["lib", 2, 2, true],
      ["util", 3, 3, false],
      ["docs", 4, 1, false],
      ["readme", 5, 1, false],
    ],
  );
  assert.ok(Object.isFrozen(rows[0]));
});

test("derives tri-state values with inherited and aggregated cascade checks", () => {
  const index = indexTree(files, accessors);
  const seeded = deriveTreeCheckedStates(index, new Set(["src"]), true);
  assert.equal(seeded.get("util"), "checked", "a checked parent implies its subtree");
  assert.equal(seeded.get("src"), "checked");
  const partial = deriveTreeCheckedStates(index, new Set(["util"]), true);
  assert.equal(partial.get("lib"), "checked");
  assert.equal(partial.get("src"), "mixed");
  assert.equal(partial.get("docs"), "unchecked");
  const independent = deriveTreeCheckedStates(index, new Set(["util"]), false);
  assert.equal(independent.get("lib"), "unchecked");
});

test("normalizes and toggles checked keys under both propagation policies", () => {
  const index = indexTree(files, accessors);
  const never = () => false;
  assert.deepEqual(normalizeTreeChecked(index, new Set(["lib"]), true), ["lib", "util"]);
  assert.deepEqual(normalizeTreeChecked(index, new Set(["lib", "ghost"]), false), ["lib"]);
  const all = toggleTreeChecked(index, new Set(), "src", { cascade: true, isDisabled: never });
  assert.deepEqual(all, ["src", "app", "lib", "util"]);
  assert.deepEqual(
    toggleTreeChecked(index, new Set(all), "app", { cascade: true, isDisabled: never }),
    ["lib", "util"],
  );
  assert.deepEqual(
    toggleTreeChecked(index, new Set(), "src", {
      cascade: true,
      isDisabled: (key) => key === "app",
    }),
    ["lib", "util"],
    "disabled descendants keep their value, leaving the parent mixed",
  );
  assert.deepEqual(
    toggleTreeChecked(index, new Set(), "src", { cascade: false, isDisabled: never }),
    ["src"],
  );
});

test("creates DOM-safe id segments that keep string and number keys distinct", () => {
  assert.equal(getTreeKeyIdSegment("src"), "key-src");
  assert.equal(getTreeKeyIdSegment(7), "num-7");
  assert.equal(getTreeKeyIdSegment("7"), "key-7");
  assert.match(getTreeKeyIdSegment("a b/c"), /^key-a-b-c-[a-z0-9]+$/);
  assert.match(getTreeKeyIdSegment(-1.5), /^num-[a-z0-9]+$/);
  assert.notEqual(getTreeKeyIdSegment("a b"), getTreeKeyIdSegment("a/b"));
});
