import assert from "node:assert/strict";
import { test } from "vitest";
import { h, nextTick, ref } from "vue";

import {
  dispatchKey,
  findNodes,
  firstChild,
  mountComponent,
  mountFresco,
} from "../testing/index.js";
import { Code, createCodeBuffer, type CodeSelection } from "./Code.js";

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
  const gutters = findNodes(
    root,
    (node) =>
      typeof node.props["aria-label"] === "string" &&
      String(node.props["aria-label"]).startsWith("Line "),
  );

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

test("streams bounded content and retains only viewport rows", async () => {
  const buffer = createCodeBuffer("zero\none", { maxLines: 3 });
  const mounted = mountComponent(Code, {
    buffer,
    lineNumbers: false,
    border: "none",
    offset: 1,
    viewportSize: 1,
    overscan: 1,
  });
  buffer.append("\ntwo\nthree");
  await nextTick();

  assert.equal(buffer.content.value, "one\ntwo\nthree");
  const rows = findNodes(mounted.root, (node) => node.props["data-code-line"] !== undefined);
  assert.equal(rows.length, 3);
  assert.deepEqual(
    rows.map((node) => node.props["data-code-line"]),
    [0, 1, 2],
  );
  assert.equal(rows.filter((node) => node.props["aria-hidden"] === false).length, 1);
  mounted.unmount();
});

test("selection is grapheme-safe and syntax diagnostics preserve provider styling", async () => {
  const selection = ref<CodeSelection>({
    anchor: { line: 0, column: 0 },
    focus: { line: 0, column: 0 },
  });
  const mounted = mountFresco(() =>
    h(Code, {
      code: "const 👍🏽 = value",
      focused: true,
      border: "none",
      selection: selection.value,
      selectionBg: "magenta",
      diagnostics: [{ line: 0, startColumn: 6, endColumn: 7, message: "rename symbol" }],
      syntaxProvider: {
        tokenize: (line: string) => [
          { text: line.slice(0, 5), fg: "cyan", bold: true },
          { text: line.slice(5) },
        ],
      },
      "onUpdate:selection": (next: CodeSelection) => {
        selection.value = next;
      },
    }),
  );

  await dispatchKey({ key: "right" });
  await dispatchKey({ key: "end" });
  await dispatchKey({ key: "left", shift: true });
  assert.equal(selection.value.focus.column, 14, "navigation counts the emoji as one grapheme");
  assert.equal(
    findNodes(mounted.root, (node) => node.type === "text" && node.props.bg === "magenta").length,
    1,
  );
  assert.equal(
    findNodes(
      mounted.root,
      (node) => node.type === "text" && node.text === "const" && node.props.fg === "cyan",
    ).length,
    1,
  );
  assert.equal(
    findNodes(mounted.root, (node) => node.type === "text" && node.props.underline === true).length,
    1,
  );
  mounted.unmount();
});

test("large buffers render work proportional to the viewport", () => {
  const code = Array.from({ length: 100_000 }, (_, index) => `line ${index}`).join("\n");
  const mounted = mountComponent(Code, {
    code,
    offset: 50_000,
    viewportSize: 6,
    overscan: 2,
  });
  const rows = findNodes(mounted.root, (node) => node.props["data-code-line"] !== undefined);
  assert.equal(rows.length, 10);
  assert.equal(rows.at(0)?.props["data-code-line"], 49_998);
  assert.equal(rows.at(-1)?.props["data-code-line"], 50_007);
  mounted.unmount();
});

test("can hide the gutter without changing code rows", () => {
  const mounted = mountComponent(Code, {
    code: "one\ntwo",
    lineNumbers: false,
    border: "none",
  });
  const root = firstChild(mounted);

  assert.equal(
    findNodes(
      root,
      (node) =>
        typeof node.props["aria-label"] === "string" &&
        String(node.props["aria-label"]).startsWith("Line "),
    ).length,
    0,
  );
  assert.deepEqual(
    findNodes(root, (node) => node.type === "text" && typeof node.text === "string").map(
      (node) => node.text,
    ),
    ["one", "two"],
  );
  mounted.unmount();
});
