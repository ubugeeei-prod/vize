import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";

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
