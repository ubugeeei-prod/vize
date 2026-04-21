import assert from "node:assert/strict";
import test from "node:test";

await import("./syntax-highlight.js");

const syntax = globalThis.__vizeDocsSyntax;

void test("normalizeLanguage resolves docs aliases", () => {
  assert.equal(syntax.normalizeLanguage("ts"), "typescript");
  assert.equal(syntax.normalizeLanguage("js"), "javascript");
  assert.equal(syntax.normalizeLanguage("sh"), "bash");
  assert.equal(syntax.normalizeLanguage("cli"), "bash");
  assert.equal(syntax.normalizeLanguage("art-vue"), "art-vue");
  assert.equal(syntax.displayLanguage("ts"), "TypeScript");
  assert.equal(syntax.displayLanguage("js"), "JavaScript");
  assert.equal(syntax.displayLanguage("bash"), "sh");
  assert.equal(syntax.normalizeLanguage("nix"), "nix");
  assert.equal(syntax.displayLanguage("nix"), "Nix");
});

void test("createHighlightedHtml highlights vue directives and strings", () => {
  const html = syntax.createHighlightedHtml('<div v-if="ready">{{ count }}</div>', "vue");

  assert.match(html, /v-code__tag/);
  assert.match(html, /v-code__directive/);
  assert.match(html, /v-code__string/);
  assert.match(html, /v-code__delimiter/);
});

void test("createHighlightedHtml highlights bash commands and flags", () => {
  const html = syntax.createHighlightedHtml(
    "vp install -D @vizejs/vite-plugin\ncargo install vize\nvize check --profile src\n$ nix develop",
    "bash",
  );

  assert.match(html, /v-code__command/);
  assert.match(html, /v-code__property/);
  assert.match(html, />vp</);
  assert.match(html, />cargo</);
  assert.match(html, />install</);
  assert.match(html, />check</);
  assert.match(html, />develop</);
  assert.match(html, />vize</);
  assert.match(html, />nix</);
  assert.match(html, /@vizejs\/vite-plugin/);
});

void test("createHighlightedHtml highlights json keys and values", () => {
  const html = syntax.createHighlightedHtml('{"preset":"opinionated","lint":true}', "json");

  assert.match(html, /v-code__attribute/);
  assert.match(html, /v-code__string/);
  assert.match(html, /v-code__boolean/);
});

void test("createHighlightedHtml highlights nix expressions", () => {
  const html = syntax.createHighlightedHtml(
    "let\n  pkgs = import <nixpkgs> {};\nin pkgs.hello",
    "nix",
  );

  assert.match(html, /v-code__keyword/);
  assert.match(html, /v-code__property/);
  assert.match(html, /v-code__type/);
});
