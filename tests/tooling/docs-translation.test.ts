import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";
import { createRequire } from "node:module";

import { createTranslationClient } from "../../docs/scripts/i18n/client.ts";
import { normalizeMarkdownDocument, protectMarkdown } from "../../docs/scripts/i18n/markdown.ts";
import { createDocumentTranslator } from "../../docs/scripts/i18n/translation.ts";

void test("translation normalization preserves complete fenced source and normalizes prose only", () => {
  const source = [
    "＃ Heading",
    "【guide】（guide/path）",
    "！[ image ](image.svg)",
    "-item",
    "",
    "````vue",
    "＃ Keep full-width source",
    "```",
    "【code】（unchanged）",
    "````",
    "",
    "~~~ts",
    "const source = '！[ literal ](x)';",
    "~~~",
    "",
    "＃ After fence",
  ].join("\n");
  const expected = [
    "# Heading",
    "[guide](guide/path)",
    "![image](image.svg)",
    "- item",
    "",
    "````vue",
    "＃ Keep full-width source",
    "```",
    "【code】（unchanged）",
    "````",
    "",
    "~~~ts",
    "const source = '！[ literal ](x)';",
    "~~~",
    "",
    "# After fence",
  ].join("\n");
  assert.equal(normalizeMarkdownDocument(source), expected);
});

void test("all translation providers retain inline source, authored links and Markdown structure", () => {
  const source = [
    "# Hello **world**",
    "- Keep `const x = '<tag>'` and [guide](guide/path).",
    "> Keep <Widget /> and https://example.invalid/path.",
    "| Name | Value |",
    "| --- | --- |",
    "| First | ~~old~~ and *new* |",
  ].join("\n");
  for (const provider of ["edge", "google", "argos"] as const) {
    const protectedSource = protectMarkdown(source, provider);
    assert.equal(protectedSource.restore(protectedSource.text), source, provider);
  }
});

void test("translated Japanese punctuation retains native-rendered emphasis and literal source", () => {
  const require = createRequire(new URL("../../docs/package.json", import.meta.url));
  const native = createRequire(require.resolve("@ox-content/vite-plugin"))("@ox-content/napi");
  const source = [
    "# **見出し:**Vize",
    "> **⚠️ 進行中の作業:**Vize",
    "文章**（強調）**です。文章*（斜体）*です。",
    "本文**`false`**です。",
    "`**literal:**Vize` and server_binary_mib",
    "[link](https://example.invalid/vuerend_%26_Vize) https://example.invalid/a_b_c",
    "",
    "```ts",
    "const input = '**⚠️ 進行中の作業:**Vize';",
    "```",
  ].join("\n");
  const normalized = normalizeMarkdownDocument(source);
  assert.equal(normalizeMarkdownDocument(normalized), normalized);
  assert.ok(normalized.includes("`**literal:**Vize` and server_binary_mib"));
  assert.ok(normalized.includes("[link](https://example.invalid/vuerend_%26_Vize)"));
  assert.ok(normalized.includes("https://example.invalid/a_b_c"));
  assert.ok(normalized.includes("const input = '**⚠️ 進行中の作業:**Vize';"));
  const result = native.transform(normalized, {});
  assert.deepEqual(result.errors, []);
  assert.match(result.html, /<strong>見出し:<\/strong> Vize/);
  assert.match(result.html, /<strong>⚠️ 進行中の作業:<\/strong> Vize/);
  assert.match(result.html, /文章 <strong>（強調）<\/strong> です/);
  assert.match(result.html, /文章 <em>（斜体）<\/em> です/);
  assert.match(result.html, /本文 <strong><code>false<\/code><\/strong> です/);
  for (const control of [
    "**Warning:** Vize",
    "**Aviso:** Vize",
    "**提示：** Vize",
    "**注意:** Vize",
  ]) {
    assert.equal(normalizeMarkdownDocument(control), control);
    assert.match(native.transform(control, {}).html, /<strong>/);
  }
});

void test("document translation conserves complete YAML values, entry links and fenced bodies", async () => {
  const translate = createDocumentTranslator("google", async (text) =>
    text.replaceAll("Hello", "Bonjour"),
  );
  const source = [
    "---",
    "layout: entry",
    "title: Hello",
    "description: Hello reader",
    "flag: true",
    "count: 7",
    "nothing: null",
    "hero:",
    "  actions:",
    "    - text: Hello guide",
    "      link: guide/intro",
    "    - text: Hello outside",
    "      link: https://example.invalid",
    "features:",
    "  - title: Hello feature",
    "    details: Hello details",
    "    link: guide/features",
    "  - title: Already localized",
    "    link: ja/guide/ready",
    "  - title: Absolute",
    "    link: /guide/root",
    "  - title: Fragment",
    "    link: '#keep'",
    "---",
    "# Hello world",
    "",
    "```vue",
    "<template>Hello unchanged</template>",
    "```",
    "",
  ].join("\n");
  const output = await translate(source, "ja", "index.md");
  const match = output.match(/^---\n([\s\S]*?)\n---\n([\s\S]*)$/);
  assert.ok(match);
  const frontmatter: unknown = parse(match[1]);
  assert.deepEqual(frontmatter, {
    layout: "entry",
    title: "Bonjour",
    description: "Bonjour reader",
    flag: true,
    count: 7,
    nothing: null,
    hero: {
      actions: [
        { text: "Bonjour guide", link: "ja/guide/intro" },
        { text: "Bonjour outside", link: "https://example.invalid" },
      ],
    },
    features: [
      { title: "Bonjour feature", details: "Bonjour details", link: "ja/guide/features" },
      { title: "Already localized", link: "ja/guide/ready" },
      { title: "Absolute", link: "/guide/root" },
      { title: "Fragment", link: "#keep" },
    ],
  });
  assert.equal(
    match[2],
    [
      "<!-- Generated translation; source: index.md -->",
      "",
      "# Bonjour world",
      "",
      "```vue",
      "<template>Hello unchanged</template>",
      "```",
      "",
    ].join("\n"),
  );
});

void test("Edge batches retain locale, whole marker ownership and duplicate request caching offline", async () => {
  const originalFetch = globalThis.fetch;
  const requests: Array<{ url: string; body: string }> = [];
  globalThis.fetch = async (input, init) => {
    const url = typeof input === "string" ? input : input instanceof URL ? input.href : input.url;
    if (url === "https://edge.microsoft.com/translate/auth") return new Response("offline-token");
    assert.ok(url.startsWith("https://api-edge.cognitive.microsofttranslator.com/translate?"));
    assert.equal(new URL(url).searchParams.get("to"), "zh-Hans");
    assert.deepEqual(init?.headers, {
      authorization: "Bearer offline-token",
      "content-type": "application/json",
    });
    assert.equal(typeof init?.body, "string");
    const body = init.body as string;
    requests.push({ url, body });
    const texts = JSON.parse(body) as Array<{ Text: string }>;
    return Response.json(
      [...texts].reverse().map(({ Text }) => ({ translations: [{ text: Text }] })),
    );
  };
  try {
    const client = createTranslationClient("edge", undefined);
    assert.deepEqual(
      await Promise.all([
        client.translate("first complete source", "zh-CN"),
        client.translate("second complete source", "zh-CN"),
        client.translate("first complete source", "zh-CN"),
      ]),
      ["first complete source", "second complete source", "first complete source"],
    );
    assert.equal(requests.length, 1);
    assert.deepEqual(JSON.parse(requests[0].body), [
      { Text: '<span class="notranslate">VIZEBATCH000000000</span>\nfirst complete source' },
      { Text: '<span class="notranslate">VIZEBATCH000000001</span>\nsecond complete source' },
    ]);
    client.close();
  } finally {
    globalThis.fetch = originalFetch;
  }
});
