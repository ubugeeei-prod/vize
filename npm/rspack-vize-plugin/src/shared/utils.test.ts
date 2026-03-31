import { describe, test } from "node:test";
import assert from "node:assert/strict";
import {
  extractCustomBlocks,
  extractStyleBlocks,
  collectTemplateAssetUrls,
  isImportableUrl,
  stripCssCommentsForScoped,
} from "./utils.js";

void describe("extractCustomBlocks", () => {
  void test("extracts a simple <i18n> custom block", () => {
    const source = `
<template><div>hello</div></template>
<script setup>console.log("hi")</script>
<i18n lang="json">{ "en": { "hello": "Hello" } }</i18n>
`;
    const blocks = extractCustomBlocks(source);
    assert.equal(blocks.length, 1);
    assert.equal(blocks[0].type, "i18n");
    assert.equal(blocks[0].attrs.lang, "json");
    assert.equal(blocks[0].content.trim(), '{ "en": { "hello": "Hello" } }');
    assert.equal(blocks[0].index, 0);
  });

  void test("does not treat <template #prefix> as a custom block", () => {
    const source = `
<template>
  <el-input>
    <template #prefix>icon</template>
    <template #default>
      <span>text</span>
    </template>
  </el-input>
</template>
<script setup>console.log("hi")</script>
`;
    const blocks = extractCustomBlocks(source);
    assert.equal(blocks.length, 0, "nested <template> slots must not become custom blocks");
  });

  void test("does not treat <template #default> as a custom block", () => {
    const source = `
<template>
  <div>
    <template #default>
      <p>default slot content</p>
    </template>
  </div>
</template>
<script>export default {}</script>
`;
    const blocks = extractCustomBlocks(source);
    assert.equal(blocks.length, 0);
  });

  void test("does not generate type=div or type=el-form-item blocks", () => {
    const source = `
<template>
  <div class="wrapper">
    <el-form-item label="Name">
      <el-input v-model="name">
        <template #prefix>
          <span class="icon">@</span>
        </template>
      </el-input>
    </el-form-item>
    <el-form-item label="Email">
      <span>test</span>
    </el-form-item>
  </div>
</template>
<script setup lang="ts">
import { ref } from 'vue'
const name = ref('')
</script>
<style scoped>
.wrapper { padding: 16px; }
</style>
`;
    const blocks = extractCustomBlocks(source);
    assert.equal(blocks.length, 0, "HTML tags inside <template> must not become custom blocks");
  });

  void test("extracts multiple custom blocks", () => {
    const source = `
<template><div>hello</div></template>
<script setup>console.log("hi")</script>
<i18n>{ "en": {} }</i18n>
<docs>## Usage</docs>
`;
    const blocks = extractCustomBlocks(source);
    assert.equal(blocks.length, 2);
    assert.equal(blocks[0].type, "i18n");
    assert.equal(blocks[0].index, 0);
    assert.equal(blocks[1].type, "docs");
    assert.equal(blocks[1].index, 1);
  });

  void test("extracts custom block with src attribute", () => {
    const source = `
<template><div>hello</div></template>
<script setup>console.log("hi")</script>
<i18n src="./locales.json" lang="json"></i18n>
`;
    const blocks = extractCustomBlocks(source);
    assert.equal(blocks.length, 1);
    assert.equal(blocks[0].type, "i18n");
    assert.equal(blocks[0].src, "./locales.json");
    assert.equal(blocks[0].attrs.lang, "json");
  });

  void test("handles SFC with no custom blocks", () => {
    const source = `
<template><div>hello</div></template>
<script setup>console.log("hi")</script>
<style scoped>.root { color: red; }</style>
`;
    const blocks = extractCustomBlocks(source);
    assert.equal(blocks.length, 0);
  });

  void test("handles deeply nested same-name tags in template", () => {
    const source = `
<template>
  <div>
    <template v-if="show">
      <template v-for="item in items" :key="item.id">
        <span>{{ item.name }}</span>
      </template>
    </template>
  </div>
</template>
<script setup>
import { ref } from 'vue'
const show = ref(true)
const items = ref([])
</script>
<i18n>{ "en": {} }</i18n>
`;
    const blocks = extractCustomBlocks(source);
    assert.equal(blocks.length, 1);
    assert.equal(blocks[0].type, "i18n");
  });

  void test("handles self-closing custom block", () => {
    const source = `
<template><div>hello</div></template>
<script setup>console.log("hi")</script>
<i18n src="./en.json" />
`;
    const blocks = extractCustomBlocks(source);
    assert.equal(blocks.length, 1);
    assert.equal(blocks[0].type, "i18n");
    assert.equal(blocks[0].src, "./en.json");
    assert.equal(blocks[0].content, "");
  });

  void test("ignores HTML comments", () => {
    const source = `
<!-- This is a comment -->
<template><div>hello</div></template>
<script setup>console.log("hi")</script>
<!-- <custom>should not match</custom> -->
<i18n>{ "en": {} }</i18n>
`;
    const blocks = extractCustomBlocks(source);
    assert.equal(blocks.length, 1);
    assert.equal(blocks[0].type, "i18n");
  });
});

void describe("extractStyleBlocks", () => {
  void test("extracts plain CSS style block", () => {
    const source = `
<template><div>hello</div></template>
<script setup>console.log("hi")</script>
<style>.root { color: red; }</style>
`;
    const blocks = extractStyleBlocks(source);
    assert.equal(blocks.length, 1);
    assert.equal(blocks[0].scoped, false);
    assert.equal(blocks[0].lang, null);
  });

  void test("extracts scoped and module style blocks", () => {
    const source = `
<template><div>hello</div></template>
<style module>.root { color: blue; }</style>
<style scoped>.note { color: red; }</style>
`;
    const blocks = extractStyleBlocks(source);
    assert.equal(blocks.length, 2);
    assert.equal(blocks[0].module, true);
    assert.equal(blocks[0].scoped, false);
    assert.equal(blocks[1].module, false);
    assert.equal(blocks[1].scoped, true);
  });

  void test("extracts named module style blocks", () => {
    const source = `
<template><div>hello</div></template>
<style module="$styleCrit">.root { color: blue; }</style>
<style module="tokens">.note { color: red; }</style>
`;
    const blocks = extractStyleBlocks(source);
    assert.equal(blocks.length, 2);
    assert.equal(blocks[0].module, "$styleCrit");
    assert.equal(blocks[1].module, "tokens");
  });
});

void describe("stripCssCommentsForScoped", () => {
  void test("removes block comments and preserves newlines", () => {
    const input = `.a { color: red; }\n/* :deep(.x) */\n.b { color: blue; }`;
    const output = stripCssCommentsForScoped(input);

    assert.equal(output.includes(":deep("), false);
    assert.equal(output.split("\n").length, input.split("\n").length);
    assert.equal(output.includes(".a { color: red; }"), true);
    assert.equal(output.includes(".b { color: blue; }"), true);
  });

  void test("keeps comment-like text inside strings", () => {
    const input = `.a::before { content: "/* :deep(.x) */"; }`;
    const output = stripCssCommentsForScoped(input);

    assert.equal(output.includes('content: "/* :deep(.x) */"'), true);
  });
});

// ============================================================================
// isImportableUrl
// ============================================================================

void describe("isImportableUrl", () => {
  void test("relative paths are importable", () => {
    assert.equal(isImportableUrl("./logo.png"), true);
    assert.equal(isImportableUrl("../assets/bg.jpg"), true);
  });

  void test("alias paths are importable", () => {
    assert.equal(isImportableUrl("@/assets/logo.svg"), true);
    assert.equal(isImportableUrl("~/images/hero.webp"), true);
    assert.equal(isImportableUrl("~bootstrap/img/flag.png"), true);
  });

  void test("external URLs are not importable", () => {
    assert.equal(isImportableUrl("https://cdn.example.com/img.png"), false);
    assert.equal(isImportableUrl("http://example.com/img.png"), false);
    assert.equal(isImportableUrl("//cdn.example.com/img.png"), false);
  });

  void test("data URIs are not importable", () => {
    assert.equal(isImportableUrl("data:image/png;base64,abc123"), false);
  });

  void test("empty string is not importable", () => {
    assert.equal(isImportableUrl(""), false);
  });
});

// ============================================================================
// collectTemplateAssetUrls
// ============================================================================

void describe("collectTemplateAssetUrls", () => {
  void test("collects src from img tag", () => {
    const source = `
<template>
  <div>
    <img src="./logo.png" alt="logo" />
  </div>
</template>
<script setup></script>
`;
    const result = collectTemplateAssetUrls(source);
    assert.equal(result.length, 1);
    assert.equal(result[0].url, "./logo.png");
    assert.equal(result[0].varName, "_imports_0");
  });

  void test("collects src from img tag — alias path", () => {
    const source = `
<template>
  <img src="@/assets/logo.svg" />
</template>
`;
    const result = collectTemplateAssetUrls(source);
    assert.equal(result.length, 1);
    assert.equal(result[0].url, "@/assets/logo.svg");
  });

  void test("deduplicates repeated URLs", () => {
    const source = `
<template>
  <div>
    <img src="./a.png" />
    <img src="./a.png" />
    <img src="./b.png" />
  </div>
</template>
`;
    const result = collectTemplateAssetUrls(source);
    assert.equal(result.length, 2);
    assert.equal(result[0].url, "./a.png");
    assert.equal(result[1].url, "./b.png");
  });

  void test("collects poster from video tag", () => {
    const source = `
<template>
  <video src="./video.mp4" poster="./poster.jpg"></video>
</template>
`;
    const result = collectTemplateAssetUrls(source);
    const urls = result.map((r) => r.url);
    assert.ok(urls.includes("./video.mp4"), "should include src");
    assert.ok(urls.includes("./poster.jpg"), "should include poster");
  });

  void test("ignores dynamic bindings (v-bind / :attr)", () => {
    const source = `
<template>
  <img :src="dynamicSrc" />
  <img v-bind:src="anotherDynamic" />
</template>
`;
    const result = collectTemplateAssetUrls(source);
    assert.equal(result.length, 0, "dynamic bindings must not be collected");
  });

  void test("ignores external URLs", () => {
    const source = `
<template>
  <img src="https://cdn.example.com/logo.png" />
</template>
`;
    const result = collectTemplateAssetUrls(source);
    assert.equal(result.length, 0, "external URLs must not be collected");
  });

  void test("returns empty when transformAssetUrls is false", () => {
    const source = `
<template>
  <img src="./logo.png" />
</template>
`;
    const result = collectTemplateAssetUrls(source, false);
    assert.equal(result.length, 0);
  });

  void test("supports custom tag/attribute map", () => {
    const source = `
<template>
  <my-image data-src="./hero.webp" />
  <img src="./ignored.png" />
</template>
`;
    const result = collectTemplateAssetUrls(source, {
      "my-image": ["data-src"],
    });
    assert.equal(result.length, 1);
    assert.equal(result[0].url, "./hero.webp");
  });

  void test("returns empty when SFC has no template block", () => {
    const source = `<script setup>const x = 1</script>`;
    const result = collectTemplateAssetUrls(source);
    assert.equal(result.length, 0);
  });

  void test("collects href from image/use SVG elements", () => {
    const source = `
<template>
  <svg>
    <image href="./sprite.svg" />
    <use href="./icon.svg#arrow" />
  </svg>
</template>
`;
    const result = collectTemplateAssetUrls(source);
    const urls = result.map((r) => r.url);
    assert.ok(urls.includes("./sprite.svg"), "should include image href");
    assert.ok(urls.includes("./icon.svg#arrow"), "should include use href with fragment");
  });

  void test("URL with hash fragment is collected as-is (fragment split happens in generateOutput)", () => {
    const source = `
<template>
  <svg>
    <use href="./icons.svg#home" />
    <use href="./icons.svg#settings" />
  </svg>
</template>
`;
    const result = collectTemplateAssetUrls(source);
    assert.equal(result.length, 2);
    assert.equal(result[0].url, "./icons.svg#home");
    assert.equal(result[1].url, "./icons.svg#settings");
  });

  void test("tilde module path is rewritten (~ stripped in import)", () => {
    const source = `
<template>
  <img src="~bootstrap/dist/img/flag.png" />
</template>
`;
    const result = collectTemplateAssetUrls(source);
    assert.equal(result.length, 1);
    assert.equal(result[0].url, "~bootstrap/dist/img/flag.png");
    // The var name is available; stripping the ~ happens in generateOutput
  });
});
