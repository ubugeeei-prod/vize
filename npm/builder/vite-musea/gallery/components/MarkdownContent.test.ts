import test from "node:test";
import assert from "node:assert/strict";
import { h } from "vue";
import { renderToString } from "vue/server-renderer";
import MarkdownContent from "./MarkdownContent";

void test("renders documentation without executable source HTML or unsafe URLs", async () => {
  const markdown = [
    "<img src=x onerror=alert(1)>",
    "[unsafe](javascript:alert(1))",
    "![unsafe image](data:text/html,evil)",
    "[safe](https://example.com/docs)",
  ].join("\n\n");
  const html = await renderToString(h(MarkdownContent, { markdown }));

  assert.doesNotMatch(html, /<img src="x"/);
  assert.doesNotMatch(html, /href="javascript:/);
  assert.doesNotMatch(html, /src="data:/);
  assert.match(html, /&lt;img src=x onerror=alert\(1\)&gt;/);
  assert.match(html, /<a href="https:\/\/example.com\/docs" rel="noreferrer">safe<\/a>/);
});

void test("keeps documentation structure and escapes code", async () => {
  const markdown =
    "# Title\n\n- [x] Done\n\n| A | B |\n| - | - |\n| one | two |\n\n" +
    "```js\nconst html = '<script>alert(1)</script>';\n```";
  const html = await renderToString(h(MarkdownContent, { markdown }));

  assert.match(html, /<h1>Title<\/h1>/);
  assert.match(html, /<input type="checkbox" checked disabled>/);
  assert.match(html, /<table>/);
  assert.match(html, /&lt;script&gt;alert\(1\)&lt;\/script&gt;/);
  assert.doesNotMatch(html, /<script>/);
});
