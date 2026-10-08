import assert from "node:assert/strict";
import test from "node:test";
import { createRequire } from "node:module";

await import("./syntax-highlight-languages.js");
await import("./syntax-highlight-core.js");
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

void test("detectLanguage resolves language-prefixed classes", () => {
  const codeElement = {
    className: "language-bash",
    getAttribute: () => null,
  };
  const preElement = {
    className: "",
    getAttribute: () => null,
  };

  assert.equal(syntax.detectLanguage(codeElement, preElement), "bash");
});

void test("createHighlightedHtml highlights vue directives and strings", () => {
  const html = syntax.createHighlightedHtml('<div v-if="ready">{{ count }}</div>', "vue");

  assert.match(html, /v-code__tag/);
  assert.match(html, /v-code__directive/);
  assert.match(html, /v-code__string/);
  assert.match(html, /v-code__delimiter/);
  assert.match(html, /v-code__delimiter">&gt;/);
});

void test("createHighlightedHtml does not treat TypeScript generics in Vue scripts as tags", () => {
  const html = syntax.createHighlightedHtml(
    '<script setup lang="ts">\nconst key: InjectionKey<Ref<Theme>> = Symbol("theme");\n</script>',
    "vue",
  );

  assert.match(html, /&lt;script/);
  assert.match(html, /&lt;\/script/);
  assert.match(html, /InjectionKey&lt;Ref&lt;Theme&gt;&gt;/);
  assert.doesNotMatch(html, /v-code__tag[^>]*>&lt;Ref/);
  assert.doesNotMatch(html, /v-code__tag[^>]*>&lt;Theme/);
});

void test("Vue style blocks use the shared language handlers", () => {
  const source =
    '<template><div class="panel">hello</div></template>\n<style scoped>\n.panel {\n  color: red;\n}\n</style>';
  const html = syntax.createHighlightedHtml(source, "vue");

  assert.match(html, /v-code__property">color<\/span>/);
  assert.match(html, /\.panel \{/);
  assert.match(html, /&lt;\/style/);
});

void test("createHighlightedHtml highlights bash commands and flags", () => {
  const html = syntax.createHighlightedHtml(
    "vp install -D @vizejs/vite-plugin\nvp exec vize check --profile src\nvp dev\n$ nix develop",
    "bash",
  );

  assert.match(html, /v-code__command/);
  assert.match(html, /v-code__property/);
  assert.match(html, />vp</);
  assert.match(html, />exec</);
  assert.match(html, />install</);
  assert.match(html, /\bcheck\b/);
  assert.match(html, /<span class="v-code__token v-code__keyword">dev<\/span>/);
  assert.match(html, />develop</);
  assert.match(html, />vize</);
  assert.match(html, />nix</);
  assert.match(html, /@vizejs\/vite-plugin/);
});

void test("createHighlightedHtml does not treat URL fragments as bash comments", () => {
  const html = syntax.createHighlightedHtml(
    "nix run github:ubugeeei-prod/vize#vize -- --help",
    "bash",
  );

  assert.doesNotMatch(html, /v-code__comment/);
  assert.match(html, /github:ubugeeei-prod\//);
  assert.match(html, /#<span class="v-code__token v-code__command">vize<\/span>/);
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

void test("native code annotations retain source, line identity, and multiline tokens", () => {
  const require = createRequire(import.meta.url);
  const native = createRequire(require.resolve("@ox-content/vite-plugin"))("@ox-content/napi");
  const { Window } = createRequire(new URL("../../tests/package.json", import.meta.url))(
    "happy-dom",
  );
  const window = new Window();
  const source = [
    '<script setup lang="ts">',
    "/* first comment line",
    "second comment line */",
    "const key: InjectionKey<Ref<Theme>> = Symbol('theme');",
    "const description = `first string line",
    "second string line`;",
    "</script>",
    "<template>",
    '  <label v-if="ready">こんにちは & {{ name }}</label>',
    "</template>",
  ].join("\n");
  const rendered = native.transform('```vue annotate="remove:2-3;add:4-6"\n' + source + "\n```", {
    codeAnnotations: true,
  });
  assert.deepEqual(rendered.errors, []);
  window.document.body.innerHTML = rendered.html;
  const code = window.document.querySelector("pre > code");
  const pre = code.closest("pre");
  const lines = Array.from(code.children);
  const attributes = lines.map((line) => line.outerHTML.split(">", 1)[0]);
  const separators = Array.from(code.childNodes).filter((node) => node.nodeType === 3);
  assert.equal(code.textContent, source + "\n");
  assert.equal(lines.length, 11);
  assert.ok(lines[1].classList.contains("ox-code-line--remove"));
  assert.ok(lines[3].classList.contains("ox-code-line--add"));

  syntax.highlightCodeElement(code);

  assert.equal(code.textContent, source + "\n");
  assert.deepEqual(Array.from(code.children), lines);
  assert.deepEqual(
    lines.map((line) => line.outerHTML.split(">", 1)[0]),
    attributes,
  );
  assert.deepEqual(
    Array.from(code.childNodes).filter((node) => node.nodeType === 3),
    separators,
  );
  assert.ok(pre.classList.contains("has-diff"));
  assert.equal(pre.dataset.language, "Vue");
  assert.ok(lines[1].querySelector(".v-code__comment"));
  assert.ok(lines[2].querySelector(".v-code__comment"));
  assert.equal(lines[3].querySelector(".v-code__tag"), null);
  assert.ok(lines[4].querySelector(".v-code__string"));
  assert.ok(lines[5].querySelector(".v-code__string"));
  assert.ok(lines[8].querySelector(".v-code__directive"));

  const highlightedHtml = code.innerHTML;
  const firstToken = code.querySelector(".v-code__token");
  syntax.highlightAll(window.document);
  assert.equal(code.innerHTML, highlightedHtml);
  assert.equal(code.querySelector(".v-code__token"), firstToken);
  window.close();
});

void test("createHighlightedHtml restores strings captured inside line comments", () => {
  const source =
    'const emit = defineEmits(["change", "unused"])\n// `unused` is never emitted; "quoted" text stays intact';
  const html = syntax.createHighlightedHtml(source, "typescript");

  assert.match(html, /`unused`/);
  assert.match(html, /"quoted"/);
  assert.equal(html.replace(/<[^>]+>/g, ""), source);
});
