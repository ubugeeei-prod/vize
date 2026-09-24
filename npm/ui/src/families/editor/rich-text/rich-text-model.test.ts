import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import { createRichTextCommands } from "./rich-text-commands.ts";
import { isBlockActive, isMarkActive, isWrappedIn } from "./rich-text-commands.ts";
import {
  comparePositions,
  emptyRichTextDoc,
  isRichTextDoc,
  normalizeInline,
  richTextDocFromText,
  sliceInline,
  validateRichTextDoc,
} from "./rich-text-model.ts";
import {
  boldSpec,
  defineRichTextSchema,
  isSafeImageUrl,
  isSafeUrl,
  paragraphSpec,
} from "./rich-text-schema.ts";
import { createRichTextState, createTransaction, redo, undo } from "./rich-text-state.ts";
import {
  apply,
  at,
  doc,
  h,
  html,
  p,
  quote,
  run,
  schema,
  state,
  t,
  text,
  ul,
} from "./rich-text-test-utils.ts";

const commands = createRichTextCommands(schema);

test("schemas validate names, a textblock default, and list pairs", () => {
  assert.equal(schema.defaultBlock, "paragraph");
  assert.throws(
    () => defineRichTextSchema({ nodes: { text: paragraphSpec() }, marks: {} }),
    /VIZE_UI_RICH_TEXT_SCHEMA/,
  );
  assert.throws(
    () => defineRichTextSchema({ nodes: {}, marks: { bold: boldSpec() } }),
    /default block/,
  );
  assert.throws(
    () =>
      defineRichTextSchema({
        nodes: { paragraph: paragraphSpec(), list: schema.nodes.bulletList },
        marks: {},
      }),
    /listItem/,
  );
});

test("URL guards allow safe schemes and relative URLs only", () => {
  for (const url of ["https://a.test", "mailto:a@b.c", "/x", "#top", "page.html", "tel:+1"])
    assert.ok(isSafeUrl(url), url);
  for (const url of ["javascript:alert(1)", " JAVASCRIPT:x", "data:text/html,x", "vbscript:x"])
    assert.ok(!isSafeUrl(url), url);
  assert.ok(isSafeImageUrl("data:image/png;base64,AAAA"));
  assert.ok(!isSafeImageUrl("data:image/svg+xml,<svg>"));
});

test("documents are validated structurally against the schema", () => {
  assert.equal(validateRichTextDoc(schema, doc(p("a"))), null);
  assert.match(
    validateRichTextDoc(schema, { type: "doc", content: [] }) ?? "",
    /at least one block/,
  );
  assert.match(
    validateRichTextDoc(schema, {
      type: "doc",
      content: [{ type: "image", attrs: {}, content: [] }],
    }) ?? "",
    /not a block/,
  );
  assert.match(
    validateRichTextDoc(schema, {
      type: "doc",
      content: [{ type: "bulletList", attrs: {}, content: [p("x")] }],
    }) ?? "",
    /list items/,
  );
  assert.match(
    validateRichTextDoc(schema, {
      type: "doc",
      content: [{ type: "codeBlock", attrs: { language: null }, content: [t("x", "bold")] }],
    }) ?? "",
    /unmarked/,
  );
  assert.equal(isRichTextDoc(schema, emptyRichTextDoc(schema)), true);
  assert.equal(richTextDocFromText(schema, "a\nb").content.length, 2);
});

test("inline helpers merge equal marks and slice by offsets", () => {
  assert.deepEqual(normalizeInline([t("a", "bold"), t("b", "bold"), t(""), t("c")]), [
    t("ab", "bold"),
    t("c"),
  ]);
  assert.deepEqual(sliceInline([t("hello", "bold"), t(" world")], 3, 8), [
    t("lo", "bold"),
    t(" wo"),
  ]);
  assert.ok(comparePositions(at([0], 5), at([1], 0)) < 0);
  assert.ok(comparePositions(at([1, 0, 0], 0), at([1], 3)) > 0);
});

test("typing inserts text with active and stored marks", () => {
  let current = state(doc(p("ab", t("cd", "bold"))), at([0], 4));
  current = apply(current, commands.insertText("!"));
  assert.equal(html(current), "<p>ab<strong>cd!</strong></p>");
  current = apply(current, commands.toggleMark("bold"));
  assert.deepEqual(current.storedMarks, []);
  current = apply(current, commands.insertText("?"));
  assert.equal(html(current), "<p>ab<strong>cd!</strong>?</p>");
});

test("links are not inclusive: typing after a link leaves it", () => {
  let current = state(doc(p("x")), at([0], 1));
  current = apply(current, commands.setLink("https://vize.dev", " site"));
  current = apply(current, commands.insertText("!"));
  assert.equal(
    html(current),
    '<p>x<a href="https://vize.dev" rel="noopener noreferrer"> site</a>!</p>',
  );
  const unsafe = run(
    state(doc(p("x")), at([0], 0), at([0], 1)),
    commands.setLink("javascript:alert(1)"),
  );
  assert.ok(unsafe);
  assert.equal(
    html(unsafe),
    '<p><a rel="noopener noreferrer">x</a></p>',
    "unsafe hrefs never render",
  );
});

test("marks toggle across multi-block selections and report activity", () => {
  let current = state(doc(p("one"), p("two")), at([0], 1), at([1], 2));
  assert.equal(isMarkActive(current, "bold"), false);
  current = apply(current, commands.toggleMark("bold"));
  assert.equal(html(current), "<p>o<strong>ne</strong></p><p><strong>tw</strong>o</p>");
  assert.equal(isMarkActive(current, "bold"), true);
  current = apply(current, commands.toggleMark("bold"));
  assert.equal(html(current), "<p>one</p><p>two</p>");
});

test("block types change, toggle back, and code blocks strip marks", () => {
  let current = state(doc(p("a", t("b", "bold"))), at([0], 1));
  current = apply(current, commands.toggleBlockType("heading", { level: 2 }));
  assert.equal(html(current), "<h2>a<strong>b</strong></h2>");
  assert.equal(isBlockActive(current, "heading", { level: 2 }), true);
  current = apply(current, commands.toggleBlockType("heading", { level: 2 }));
  assert.equal(html(current), "<p>a<strong>b</strong></p>");
  current = apply(current, commands.setBlockType("codeBlock"));
  assert.equal(html(current), "<pre>ab</pre>");
  assert.equal(run(current, commands.toggleMark("bold")), null, "marks do not apply in code");
  assert.equal(run(current, commands.setBlockType("codeBlock")), null, "no-op when already active");
});

test("Enter splits blocks, leaves headings as paragraphs, and inserts newlines in code", () => {
  let current = state(doc(p("hello")), at([0], 2));
  current = apply(current, commands.splitBlock);
  assert.equal(html(current), "<p>he</p><p>llo</p>");
  assert.deepEqual(current.selection.head, at([1], 0));
  current = apply(state(doc(h(1, "Title")), at([0], 5)), commands.splitBlock);
  assert.equal(html(current), "<h1>Title</h1><p></p>");
  current = apply(
    state(doc({ type: "codeBlock", attrs: { language: null }, content: [t("a")] }), at([0], 1)),
    commands.splitBlock,
  );
  assert.equal(html(current), "<pre>a\n</pre>");
});

test("Enter in lists splits items and leaves the list from an empty item", () => {
  let current = state(doc(ul([p("one")])), at([0, 0, 0], 3));
  current = apply(current, commands.splitBlock);
  assert.equal(html(current), "<ul><li><p>one</p></li><li><p></p></li></ul>");
  assert.deepEqual(current.selection.head, at([0, 1, 0], 0));
  current = apply(current, commands.splitBlock);
  assert.equal(html(current), "<ul><li><p>one</p></li></ul><p></p>");
  assert.deepEqual(current.selection.head, at([1], 0));
});

test("Backspace deletes characters, surrogate pairs, words, selections, and joins blocks", () => {
  let current = state(doc(p("a😀b")), at([0], 3));
  current = apply(current, commands.deleteBackward());
  assert.equal(text(current), "ab");
  current = apply(state(doc(p("hello big world")), at([0], 15)), commands.deleteBackward("word"));
  assert.equal(text(current), "hello big ");
  current = apply(
    state(doc(p("abc"), p("def")), at([0], 1), at([1], 2)),
    commands.deleteBackward(),
  );
  assert.equal(html(current), "<p>af</p>");
  current = apply(state(doc(p("abc"), p("def")), at([1], 0)), commands.deleteBackward());
  assert.equal(html(current), "<p>abcdef</p>");
  assert.deepEqual(current.selection.head, at([0], 3));
  current = apply(state(doc(h(1, "T")), at([0], 0)), commands.deleteBackward());
  assert.equal(html(current), "<p>T</p>", "resets a leading heading");
  assert.equal(run(state(doc(p("x")), at([0], 0)), commands.deleteBackward()), null);
});

test("Backspace at the start of a list item or quote lifts it out", () => {
  let current = state(doc(ul([p("a")], [p("b")])), at([0, 1, 0], 0));
  current = apply(current, commands.deleteBackward());
  assert.equal(
    html(current),
    "<ul><li><p>ab</p></li></ul>",
    "later items merge into the previous item",
  );
  current = apply(state(doc(ul([p("a")], [p("b")])), at([0, 0, 0], 0)), commands.deleteBackward());
  assert.equal(html(current), "<p>a</p><ul><li><p>b</p></li></ul>");
  current = apply(state(doc(quote(p("q"))), at([0, 0], 0)), commands.deleteBackward());
  assert.equal(html(current), "<p>q</p>");
});

test("Delete removes the next character or merges the next block", () => {
  let current = apply(state(doc(p("ab")), at([0], 0)), commands.deleteForward());
  assert.equal(text(current), "b");
  current = apply(state(doc(p("a"), p("b")), at([0], 1)), commands.deleteForward());
  assert.equal(html(current), "<p>ab</p>");
  assert.equal(run(state(doc(p("a")), at([0], 1)), commands.deleteForward()), null);
});

test("wrapping in lists and quotes, toggling off, and mapping the selection", () => {
  let current = state(doc(p("a"), p("b"), p("c")), at([0], 0), at([1], 1));
  current = apply(current, commands.toggleWrap("bulletList"));
  assert.equal(html(current), "<ul><li><p>a</p></li><li><p>b</p></li></ul><p>c</p>");
  assert.deepEqual(current.selection.head, at([0, 1, 0], 1));
  assert.equal(isWrappedIn(current, "bulletList"), true);
  current = apply(current, commands.toggleWrap("bulletList"));
  assert.equal(html(current), "<p>a</p><p>b</p><p>c</p>");
  current = apply(state(doc(ul([p("x")])), at([0, 0, 0], 1)), commands.wrapIn("blockquote"));
  assert.equal(
    html(current),
    "<ul><li><blockquote><p>x</p></blockquote></li></ul>",
    "a caret inside an item wraps the item's block",
  );
  current = apply(state(doc(p("n")), at([0], 0)), commands.wrapIn("orderedList", { start: 3 }));
  assert.equal(html(current), '<ol start="3"><li><p>n</p></li></ol>');
});

test("inline leaves insert at the caret with sanitized attributes", () => {
  let current = apply(
    state(doc(p("ab")), at([0], 1)),
    commands.insertInline("image", { src: "https://x/y.png", alt: "Y" }),
  );
  assert.equal(html(current), '<p>a<img src="https://x/y.png" alt="Y">b</p>');
  assert.deepEqual(current.selection.head, at([0], 2));
  current = apply(state(doc(p("ab")), at([0], 1)), commands.insertInline("hardBreak"));
  assert.equal(html(current), "<p>a<br>b</p>");
  current = apply(
    state(doc(p("a")), at([0], 1)),
    commands.insertInline("image", { src: "javascript:x" }),
  );
  assert.equal(html(current), '<p>a<img alt=""></p>');
});

test("history groups typing, separates formatting, and supports redo", () => {
  let current = state(doc(p("")), at([0], 0));
  for (const [character, time] of [
    ["a", 0],
    ["b", 100],
    ["c", 200],
  ] as const) {
    let next = current;
    commands.insertText(character)(current, (transaction) => {
      next = createTransaction(
        transaction.before,
        { doc: transaction.state.doc, selection: transaction.state.selection },
        "input",
        { time },
      ).state;
    });
    current = next;
  }
  current = apply(current, commands.selectAll);
  current = apply(current, commands.toggleMark("bold"));
  assert.equal(current.history.done.length, 2);
  current = apply(current, undo);
  assert.equal(html(current), "<p>abc</p>");
  current = apply(current, undo);
  assert.equal(html(current), "<p></p>");
  assert.equal(run(current, undo), null);
  current = apply(current, redo);
  current = apply(current, redo);
  assert.equal(html(current), "<p><strong>abc</strong></p>");
  assert.equal(run(current, redo), null);
});

test("transactions clamp selections and reject invalid documents", () => {
  const current = createRichTextState(schema, {
    doc: doc(p("ab")),
    selection: { anchor: at([0], 9), head: at([4], 0) },
  });
  assert.deepEqual(current.selection, { anchor: at([0], 2), head: at([0], 2) });
  assert.throws(
    () =>
      createTransaction(
        current,
        { doc: { type: "doc", content: [{ type: "nope", attrs: {}, content: [] }] } },
        "input",
      ),
    /VIZE_UI_RICH_TEXT_MODEL/,
  );
});

test("commands for undeclared types are rejected at runtime", () => {
  const small = defineRichTextSchema({ nodes: { paragraph: paragraphSpec() }, marks: {} });
  const smallCommands = createRichTextCommands(small);
  const smallState = createRichTextState(small);
  assert.equal(smallCommands.setLink("https://x", "x")(smallState), false);
});

test("inserting content merges edge textblocks and keeps middle blocks", () => {
  const fragment = doc(p("X"), h(2, "Mid"), p("Y"));
  const current = apply(state(doc(p("ab")), at([0], 1)), commands.insertContent(fragment));
  assert.equal(html(current), "<p>aX</p><h2>Mid</h2><p>Yb</p>");
  assert.deepEqual(current.selection.head, at([2], 1));
  const inline = apply(
    state(doc(p("ab")), at([0], 1)),
    commands.insertContent(doc(p(t("Z", "bold")))),
  );
  assert.equal(html(inline), "<p>a<strong>Z</strong>b</p>");
  const code = apply(
    state(doc({ type: "codeBlock", attrs: { language: null }, content: [] }), at([0], 0)),
    commands.insertContent(fragment),
  );
  assert.equal(html(code), "<pre>X\nMid\nY</pre>", "code blocks receive plain text");
  const listed = apply(
    state(doc(ul([p("i")])), at([0, 0, 0], 1)),
    commands.insertContent(doc(p("1"), p("2"))),
  );
  assert.equal(html(listed), "<ul><li><p>i1</p></li><li><p>2</p></li></ul>");
});
