import assert from "node:assert/strict";
import test from "node:test";
import {
  applyOpenGraphMetadata,
  assertPngDimensions,
  pageMetadata,
  pageRoute,
} from "../../docs/theme/open-graph.ts";

const english = {
  title: "Getting Started",
  socialTitle: "Getting Started - Vize",
  description: "Add Vize to Vue with Vite+, one config file, and copyable commands.",
  paragraph: "Add Vize to your Vue 3 app with Vite+.",
  language: "en",
};

await test("authored guide text and localized route identity remain independent", () => {
  const en = pageMetadata("/getting-started/", english, "template-a");
  const ja = pageMetadata(
    "/ja/getting-started/",
    {
      ...english,
      title: "はじめる",
      socialTitle: "はじめる - Vize",
      description: "Vite+ と1つの設定ファイルで Vue に Vize を導入する。",
      language: "ja",
    },
    "template-a",
  );
  assert.equal(en.title, "Getting Started - Vize");
  assert.equal(en.description, english.description);
  assert.equal(en.descriptionOrigin, "authored");
  assert.equal(en.props.category, "Guide");
  assert.equal(en.locale, "en_US");
  assert.equal(ja.props.category, "ガイド");
  assert.equal(ja.props.localeName, "日本語");
  assert.equal(ja.locale, "ja_JP");
  assert.equal(ja.url, "https://vizejs.dev/ja/getting-started/");
  assert.notEqual(en.image, ja.image);
  assert.throws(() => pageMetadata("/ja/getting-started/", english, "a"), /HTML language/);
});

await test("rule content supplies its own missing description and long title", () => {
  const rule = pageMetadata(
    "/rules/reference/vue-component-name-in-template-casing/",
    {
      title: "vue/component-name-in-template-casing",
      socialTitle: "vue/component-name-in-template-casing - Vize",
      description: "",
      paragraph: "Enforce specific casing for component names in templates",
      language: "en",
    },
    "a",
  );
  assert.equal(rule.description, "Enforce specific casing for component names in templates");
  assert.equal(rule.descriptionOrigin, "content");
  assert.equal(rule.props.category, "Rule reference");
  assert.equal(rule.props.title, "vue/component-name-in-template-casing");
  assert.equal(rule.props.isHome, false);
  assert.equal(rule.type, "website");
});

await test("equal-title routes and changed rendering assets receive distinct image URLs", () => {
  const first = pageMetadata("/guide/configuration/", english, "a");
  const second = pageMetadata("/guide/migration/", english, "a");
  const changed = pageMetadata("/guide/configuration/", english, "changed-logo-or-css");
  assert.notEqual(first.image, second.image);
  assert.notEqual(first.image, changed.image);
  assert.equal(first.image, pageMetadata("/guide/configuration/", english, "a").image);
  assert.equal(pageRoute("index.md"), "/");
  assert.equal(pageRoute("ja/index.md"), "/ja/");
  assert.equal(pageRoute("guide/configuration.markdown"), "/guide/configuration/");
  assert.equal(pageRoute("guide/configuration.mdx"), "/guide/configuration/");
  assert.equal(pageRoute("guide/configuration/index.md"), pageRoute("guide/configuration.md"));
  assert.equal(pageMetadata("/", { ...english, title: "Vize" }, "a").props.isHome, true);
  assert.equal(
    pageMetadata("/ja/", { ...english, title: "Vize", language: "ja" }, "a").props.isHome,
    true,
  );
});

await test("social metadata replaces stale tags, preserves unrelated head and escapes content", () => {
  const page = pageMetadata(
    "/guide/configuration/",
    {
      ...english,
      socialTitle: 'Configure "Vue" & <Vize>',
      description: 'Use "props" & <components>.',
    },
    "a",
  );
  const original =
    '<!doctype html><html lang="en"><head><meta charset="utf-8"><meta property="og:image" content="old"><meta name="twitter:image" content="old"><meta name="description" content="old"></head><body><h1>Guide</h1></body></html>';
  const result = applyOpenGraphMetadata(original, page);
  assert.equal((result.match(/property="og:image" /gu) ?? []).length, 1);
  assert.equal((result.match(/name="twitter:image" /gu) ?? []).length, 1);
  assert.match(result, /Configure &quot;Vue&quot; &amp; &lt;Vize&gt;/u);
  assert.match(result, /Use &quot;props&quot; &amp; &lt;components&gt;\./u);
  assert.match(result, /<meta charset="utf-8">/u);
  assert(result.indexOf('<meta charset="utf-8">') < result.indexOf('property="og:title"'));
  assert.match(result, /<body><h1>Guide<\/h1><\/body>/u);
  assert.equal(applyOpenGraphMetadata(result, page), result);
  const body = '<body><template><meta property="og:image" content="example"></template></body>';
  assert(applyOpenGraphMetadata(`<head></head>${body}`, page).endsWith(body));
  assert.throws(() => applyOpenGraphMetadata("<body></body>", page), /no head/);
});

await test("invalid or wrong-size images fail before publication", () => {
  const header = Buffer.alloc(24);
  Buffer.from("89504e470d0a1a0a", "hex").copy(header);
  header.write("IHDR", 12);
  header.writeUInt32BE(1200, 16);
  header.writeUInt32BE(630, 20);
  assert.doesNotThrow(() => assertPngDimensions(header, "guide"));
  assert.throws(() => assertPngDimensions(Buffer.from("failed image"), "guide"), /invalid PNG/);
  header.writeUInt32BE(600, 16);
  assert.throws(() => assertPngDimensions(header, "guide"), /1200×630/u);
});
