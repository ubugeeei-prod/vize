import assert from "node:assert/strict";
import test from "node:test";

await import("./syntax-highlight.js");

const syntax = globalThis.__vizeDocsSyntax;

void test("normalizeLanguage resolves docs aliases", () => {
  assert.equal(syntax.normalizeLanguage("ts"), "typescript");
  assert.equal(syntax.normalizeLanguage("js"), "javascript");
  assert.equal(syntax.normalizeLanguage("sh"), "bash");
  assert.equal(syntax.normalizeLanguage("art-vue"), "art-vue");
});

void test("createHighlightedHtml highlights vue directives and strings", () => {
  const html = syntax.createHighlightedHtml('<div v-if="ready">{{ count }}</div>', "vue");

  assert.match(html, /v-code__tag/);
  assert.match(html, /v-code__directive/);
  assert.match(html, /v-code__string/);
  assert.match(html, /v-code__delimiter/);
});

void test("createHighlightedHtml highlights bash commands and flags", () => {
  const html = syntax.createHighlightedHtml("pnpm add -D vize", "bash");

  assert.match(html, /v-code__command/);
  assert.match(html, /v-code__property/);
});

void test("createHighlightedHtml highlights json keys and values", () => {
  const html = syntax.createHighlightedHtml('{"preset":"opinionated","lint":true}', "json");

  assert.match(html, /v-code__attribute/);
  assert.match(html, /v-code__string/);
  assert.match(html, /v-code__boolean/);
});
