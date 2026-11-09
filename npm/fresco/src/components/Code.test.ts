import assert from "node:assert/strict";
import { test } from "vitest";

import { findNodes, firstChild, mountComponent } from "../testing/index.js";
import { Code } from "./Code.js";

test("composes aligned public line-number gutters", () => {
  const mounted = mountComponent(Code, {
    code: "alpha\n\nomega",
    language: "ts",
    startLine: 8,
    highlightLines: [9],
    border: "none",
    lineNumberFg: "cyan",
  });
  const root = firstChild(mounted);
  const gutters = findNodes(root, (node) => typeof node.props["aria-label"] === "string");

  assert.deepEqual(
    gutters.map((node) => ({
      text: node.props.text,
      fg: node.props.fg,
      label: node.props["aria-label"],
    })),
    [
      { text: " 8 │ ", fg: "cyan", label: "Line 8" },
      { text: " 9 │ ", fg: "cyan", label: "Line 9" },
      { text: "10 │ ", fg: "cyan", label: "Line 10" },
    ],
  );

  const highlightedEmptyLine = findNodes(
    root,
    (node) => node.type === "text" && node.text === " " && node.props.bg === "blue",
  );
  assert.equal(highlightedEmptyLine.length, 1);
  mounted.unmount();
});

test("can hide the gutter without changing code rows", () => {
  const mounted = mountComponent(Code, {
    code: "one\ntwo",
    lineNumbers: false,
    border: "none",
  });
  const root = firstChild(mounted);

  assert.equal(findNodes(root, (node) => typeof node.props["aria-label"] === "string").length, 0);
  assert.deepEqual(
    findNodes(root, (node) => node.type === "text" && typeof node.text === "string").map(
      (node) => node.text,
    ),
    ["one", "two"],
  );
  mounted.unmount();
});
