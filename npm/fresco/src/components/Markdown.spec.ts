import assert from "node:assert/strict";
import { test } from "vitest";

import { findNodes, firstChild, mountComponent } from "../testing/index.js";
import Markdown from "./FrescoMarkdown.vue";
import { parseMarkdown, renderMarkdown } from "./markdown.js";
import type { MarkdownBlockProvider } from "./markdown-types.js";

test("parses the lightweight CommonMark block contract", () => {
  const blocks = parseMarkdown(`# Heading

Paragraph with **strength**.

> quoted
> together

3. first
4. second

- [x] complete
- [ ] pending

\`\`\`ts
const answer = 42;
\`\`\`

---`);

  assert.deepEqual(
    blocks.map((block) => block.type),
    ["heading", "paragraph", "quote", "list", "list", "code", "thematic-break"],
  );
  assert.deepEqual(blocks[2], { type: "quote", line: 5, content: "quoted together" });
  assert.deepEqual(blocks[3], {
    type: "list",
    line: 8,
    ordered: true,
    start: 3,
    items: [{ content: "first" }, { content: "second" }],
  });
  assert.deepEqual(blocks[4], {
    type: "list",
    line: 11,
    ordered: false,
    items: [
      { content: "complete", checked: true },
      { content: "pending", checked: false },
    ],
  });
  assert.deepEqual(blocks[5], {
    type: "code",
    line: 14,
    info: "ts",
    content: "const answer = 42;",
  });
});

test("delegates selected blocks before the built-in renderer", () => {
  const provider: MarkdownBlockProvider = {
    render(block) {
      if (block.type !== "quote") return undefined;
      return [{ key: `note-${block.line}`, spans: [{ text: `NOTE: ${block.content}` }] }];
    },
  };

  const lines = renderMarkdown("# Title\n\n> provider", { providers: [provider] });
  assert.equal(lines[0]?.spans.map((span) => span.text).join(""), "# Title");
  assert.deepEqual(lines[1], {
    key: "note-3",
    spans: [{ text: "NOTE: provider" }],
  });
});

test("renders styled inline spans through the canonical SFC", () => {
  const mounted = mountComponent(Markdown, {
    source: "# Welcome\n\nRead [the guide](https://example.test) with `code`.",
    headingFg: "cyan",
    linkFg: "blue",
    codeFg: "yellow",
  });
  const root = firstChild(mounted);
  const textNodes = findNodes(root, (node) => typeof node.props.text === "string");

  assert.deepEqual(
    textNodes.map((node) => node.props.text),
    ["# ", "Welcome", "Read ", "the guide", " (https://example.test)", " with ", "code", "."],
  );
  assert.equal(textNodes[1]?.props.bold, true);
  assert.equal(textNodes[3]?.props.underline, true);
  assert.equal(textNodes[3]?.props.fg, "blue");
  assert.equal(textNodes[6]?.props.fg, "yellow");
  mounted.unmount();
});

function inlineSpans(source: string) {
  const lines = renderMarkdown(source, { linkFg: "blue" });
  assert.equal(lines.length, 1);
  return lines[0]?.spans;
}

test("renders inline links with and without a title", () => {
  const label = (text: string) => ({ text, fg: "blue", underline: true });
  const target = (url: string) => ({ text: ` (${url})`, dim: true });

  assert.deepEqual(inlineSpans("[guide](https://example.test/a?b=1#c)"), [
    label("guide"),
    target("https://example.test/a?b=1#c"),
  ]);
  assert.deepEqual(inlineSpans('See [the guide](/docs  "Read me") now'), [
    { text: "See " },
    label("the guide"),
    target("/docs"),
    { text: " now" },
  ]);
  assert.deepEqual(inlineSpans("[a](b)[c](d)"), [label("a"), target("b"), label("c"), target("d")]);
  assert.deepEqual(inlineSpans("[[nested](url)"), [label("[nested"), target("url")]);
  assert.deepEqual(inlineSpans("[label](one(two)"), [label("label"), target("one(two")]);
  assert.deepEqual(inlineSpans('[label](url"quoted")'), [label("label"), target('url"quoted"')]);
  assert.deepEqual(inlineSpans('[label](url\u00a0\t"title")'), [label("label"), target("url")]);
});

test("leaves bracketed text that is not a link as plain text", () => {
  for (const source of [
    "[]",
    "[](url)",
    "[label]",
    "[label] (url)",
    "[label]()",
    "[label]( url)",
    "[label](url",
    "[label](url title)",
    '[label](url "title)',
    '[label](url "title" )',
    '[label](url "a" "b")',
  ]) {
    assert.deepEqual(inlineSpans(source), [{ text: source }], source);
  }
});

test("long runs of unmatched link syntax are scanned once", () => {
  for (const source of [
    "[".repeat(40_000),
    `${"[".repeat(40_000)}]`,
    "[a](".repeat(10_000),
    `${"[a](".repeat(10_000)} )`,
    `${"[a](".repeat(10_000)}x${" ".repeat(20_000)}y`,
  ]) {
    const started = performance.now();
    const spans = inlineSpans(source);
    const elapsed = performance.now() - started;
    assert.deepEqual(spans, [{ text: source }]);
    assert.ok(elapsed < 1_000, `${source.slice(0, 8)}… took ${Math.round(elapsed)}ms`);
  }
});
