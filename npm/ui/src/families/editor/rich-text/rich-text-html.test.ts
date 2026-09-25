import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import { richTextFromHtml, richTextToHtml } from "./rich-text-html.ts";
import { defaultRichTextInputRules, insertTextWithRules } from "./rich-text-input-rules.ts";
import { defaultRichTextKeymap, richTextKeyName } from "./rich-text-keymap.ts";
import { apply, at, doc, h, html, p, schema, state, t, ul } from "./rich-text-test-utils.ts";

const rules = defaultRichTextInputRules(schema);

function typeInto(current: ReturnType<typeof state>, text: string) {
  let next = current;
  for (const character of text) next = apply(next, insertTextWithRules(character, rules));
  return next;
}

test("serializes escaped text, nested marks, and editor hooks", () => {
  const document = doc(p(t("<a&b>", "bold", "italic"), t(' "q"')), ul([p("")]));
  assert.equal(
    richTextToHtml(schema, document),
    "<p><strong><em>&lt;a&amp;b&gt;</em></strong> &quot;q&quot;</p><ul><li><p></p></li></ul>",
  );
  assert.equal(
    richTextToHtml(schema, document, { editor: true }),
    '<p data-rt-path="0" data-rt-textblock=""><strong><em>&lt;a&amp;b&gt;</em></strong> &quot;q&quot;</p>' +
      '<ul data-rt-path="1"><li data-rt-path="1.0"><p data-rt-path="1.0.0" data-rt-textblock=""><br data-rt-filler=""></p></li></ul>',
  );
});

test("parses pasted HTML into schema nodes and drops everything else", () => {
  const parsed = richTextFromHtml(
    schema,
    `<h2>Title</h2><script>alert(1)</script><p onclick="x()">Hi <b>bold <i>both</i></b>
     <a href="javascript:alert(1)">bad</a> <a href="https://ok.test">ok</a></p>
     <ul><li>one</li><li><p>two</p><ol start="4"><li>nested</li></ol></li></ul>
     <blockquote><p>quoted</p></blockquote><pre data-language="ts">let x;\n  y</pre>
     <div><span style="color:red">loose</span><img src="javascript:x"><img src="/a.png" alt="A"></div>
     <table><tr><td>cell</td></tr></table>`,
  );
  assert.equal(
    html(parsed),
    '<h2>Title</h2><p>Hi <strong>bold <em>both</em></strong> bad <a href="https://ok.test" rel="noopener noreferrer">ok</a></p>' +
      '<ul><li><p>one</p></li><li><p>two</p><ol start="4"><li><p>nested</p></li></ol></li></ul>' +
      '<blockquote><p>quoted</p></blockquote><pre data-language="ts">let x;\n  y</pre>' +
      '<p>loose<img src="/a.png" alt="A"></p><p>cell</p>',
  );
});

test("parsing an empty fragment yields one empty block", () => {
  assert.equal(html(richTextFromHtml(schema, "<script>x</script>")), "<p></p>");
});

test("block input rules turn markdown prefixes into structure", () => {
  assert.equal(html(typeInto(state(doc(p("")), at([0], 0)), "## Hi")), "<h2>Hi</h2>");
  assert.equal(
    html(typeInto(state(doc(p("")), at([0], 0)), "> q")),
    "<blockquote><p>q</p></blockquote>",
  );
  assert.equal(html(typeInto(state(doc(p("")), at([0], 0)), "- a")), "<ul><li><p>a</p></li></ul>");
  assert.equal(
    html(typeInto(state(doc(p("")), at([0], 0)), "3. a")),
    '<ol start="3"><li><p>a</p></li></ol>',
  );
  assert.equal(html(typeInto(state(doc(p("")), at([0], 0)), "```")), "<pre></pre>");
  assert.equal(
    html(typeInto(state(doc(p("x")), at([0], 1)), " # no")),
    "<p>x # no</p>",
    "only at block start",
  );
});

test("mark input rules wrap delimited text and stop extending it", () => {
  assert.equal(
    html(typeInto(state(doc(p("")), at([0], 0)), "a **b** c")),
    "<p>a <strong>b</strong> c</p>",
  );
  assert.equal(html(typeInto(state(doc(p("")), at([0], 0)), "x _i_")), "<p>x <em>i</em></p>");
  assert.equal(html(typeInto(state(doc(p("")), at([0], 0)), "`c`d")), "<p><code>c</code>d</p>");
  assert.equal(html(typeInto(state(doc(p("")), at([0], 0)), "~~s~~")), "<p><s>s</s></p>");
});

test("input rules are skipped in code blocks and undo as one step", () => {
  const code = state(
    doc({ type: "codeBlock", attrs: { language: null }, content: [] }),
    at([0], 0),
  );
  assert.equal(html(typeInto(code, "# **x**")), "<pre># **x**</pre>");
  const heading = typeInto(state(doc(p("")), at([0], 0)), "# ");
  assert.equal(html(heading), "<h1></h1>");
  const undone = apply(heading, defaultRichTextKeymap(schema)["Mod-z"] ?? (() => false));
  assert.equal(html(undone), "<p>#</p>");
});

test("key names normalize modifiers and physical digits", () => {
  const key = (init: KeyboardEventInit) => richTextKeyName(new KeyboardEvent("keydown", init));
  assert.equal(key({ key: "B", ctrlKey: true }), "Mod-b");
  assert.equal(key({ key: "z", metaKey: true, shiftKey: true }), "Mod-Shift-z");
  assert.equal(key({ key: "¡", code: "Digit1", ctrlKey: true, altKey: true }), "Mod-Alt-1");
  assert.equal(key({ key: "Enter", shiftKey: true }), "Shift-Enter");
});

test("the default keymap binds marks, headings, lists, and history", () => {
  const keymap = defaultRichTextKeymap(schema);
  let current = state(doc(p("ab")), at([0], 0), at([0], 2));
  current = apply(current, keymap["Mod-b"] ?? (() => false));
  assert.equal(html(current), "<p><strong>ab</strong></p>");
  current = apply(current, keymap["Mod-Alt-2"] ?? (() => false));
  assert.equal(html(current), "<h2><strong>ab</strong></h2>");
  current = apply(current, keymap["Mod-Shift-8"] ?? (() => false));
  assert.equal(html(current), "<ul><li><h2><strong>ab</strong></h2></li></ul>");
  current = apply(state(doc(h(1, "x")), at([0], 1)), keymap["Shift-Enter"] ?? (() => false));
  assert.equal(html(current), "<h1>x<br></h1>");
  for (const binding of [
    "Mod-z",
    "Mod-Shift-z",
    "Mod-y",
    "Enter",
    "Backspace",
    "Delete",
    "Mod-e",
    "Mod-Shift-9",
  ]) {
    assert.equal(typeof keymap[binding], "function", binding);
  }
});
