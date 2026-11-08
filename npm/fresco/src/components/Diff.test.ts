import assert from "node:assert/strict";
import { test } from "vitest";

import { findNodes, firstChild, mountComponent } from "../testing/index.js";
import { Diff } from "./Diff.js";

function textValues(root: ReturnType<typeof firstChild>): string[] {
  return findNodes(root, (node) => typeof node.props.text === "string").map(
    (node) => node.props.text as string,
  );
}

function ariaLabel(node: ReturnType<typeof firstChild>): string {
  const label = node.props["aria-label"];
  return typeof label === "string" ? label : "";
}

test("renders unified hunks with aligned old and new line numbers", () => {
  const mounted = mountComponent(Diff, {
    oldText: "one\ntwo\nthree",
    newText: "one\nTWO\nthree\nfour",
    context: 1,
  });
  const root = firstChild(mounted);

  assert.ok(textValues(root).includes("@@ -1,3 +1,4 @@"));
  assert.deepEqual(
    findNodes(root, (node) => typeof node.props["aria-label"] === "string")
      .map((node) => node.props["aria-label"])
      .filter((label) => String(label).startsWith("Line ")),
    ["Line 1", "Line 1", "Line 2", "Line 2", "Line 3", "Line 3", "Line 4"],
  );
  assert.ok(
    findNodes(root, (node) => node.props.underline === true).some(
      (node) => node.props.text === "two" && node.props.bg === "darkred",
    ),
  );
  assert.ok(
    findNodes(root, (node) => node.props.underline === true).some(
      (node) => node.props.text === "TWO" && node.props.bg === "darkgreen",
    ),
  );
  mounted.unmount();
});

test("pairs changed lines in split view", () => {
  const mounted = mountComponent(Diff, {
    oldText: "before\nshared",
    newText: "after\nshared",
    view: "split",
    context: 0,
  });
  const root = firstChild(mounted);
  const panes = findNodes(root, (node) => ariaLabel(node).endsWith("diff pane"));

  assert.equal(panes.length, 2);
  assert.deepEqual(
    panes.map((pane) => pane.props["aria-label"]),
    ["old diff pane", "new diff pane"],
  );
  assert.ok(textValues(panes[0] as (typeof panes)[number]).includes("before"));
  assert.ok(textValues(panes[1] as (typeof panes)[number]).includes("after"));
  mounted.unmount();
});

test("keeps only requested context around distant changes", () => {
  const oldLines = Array.from({ length: 20 }, (_, index) => `line ${index + 1}`);
  const newLines = [...oldLines];
  newLines[2] = "changed 3";
  newLines[17] = "changed 18";
  const mounted = mountComponent(Diff, {
    oldText: oldLines.join("\n"),
    newText: newLines.join("\n"),
    context: 1,
  });
  const root = firstChild(mounted);

  assert.equal(findNodes(root, (node) => node.props["aria-label"] === "Diff hunk").length, 2);
  assert.equal(findNodes(root, (node) => ariaLabel(node).endsWith("diff line")).length, 8);
  mounted.unmount();
});

test("bounds mounted rows for a large unchanged file", () => {
  const text = Array.from({ length: 10_000 }, (_, index) => `line ${index + 1}`).join("\n");
  const mounted = mountComponent(Diff, {
    oldText: text,
    newText: text,
    viewportStart: 5_000,
    viewportSize: 5,
    overscan: 2,
  });
  const root = firstChild(mounted);

  assert.equal(root.children.length, 9);
  assert.equal(root.props["aria-label"], "unified diff, 10000 rows");
  const labels = findNodes(root, (node) => ariaLabel(node).startsWith("Line "));
  assert.equal(labels.length, 18);
  assert.equal(labels[0]?.props["aria-label"], "Line 4999");
  assert.equal(labels.at(-1)?.props["aria-label"], "Line 5007");
  mounted.unmount();
});
