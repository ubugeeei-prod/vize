import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import { artScopeAttribute } from "../art-style.js";
import { processMuseaArtFile, reportArtStatusWarnings } from "./art-processing.js";

void test("processMuseaArtFile forwards parser diagnostics during build", async () => {
  const tempDir = await fs.promises.mkdtemp(path.join(os.tmpdir(), "musea-bad-art-"));
  const artPath = path.join(tempDir, "stories", "Broken.art.vue");

  try {
    await fs.promises.mkdir(path.dirname(artPath), { recursive: true });
    await fs.promises.writeFile(
      artPath,
      '<art><variant name="Default"><div /></variant></art>',
      "utf8",
    );

    await assert.rejects(
      processMuseaArtFile(artPath, { root: tempDir, command: "build" }),
      (error) => {
        const message = error instanceof Error ? error.message : String(error);
        assert.match(message, /\[musea\] Failed to process stories\/Broken\.art\.vue/);
        assert.match(message, /Missing required 'title' attribute in <art> block/);
        return true;
      },
    );
  } finally {
    await fs.promises.rm(tempDir, { recursive: true, force: true });
  }
});

void test("processMuseaArtFile scopes art style blocks onto variant elements", async () => {
  const tempDir = await fs.promises.mkdtemp(path.join(os.tmpdir(), "musea-scoped-art-"));
  const artPath = path.join(tempDir, "my-card.art.vue");

  try {
    await fs.promises.writeFile(
      artPath,
      `<art title="MyCard">
  <variant name="Default" default>
    <div class="wrap"><span>content</span></div>
  </variant>
</art>
<style scoped>
.wrap { padding: 16px; }
</style>
`,
      "utf8",
    );

    const info = await processMuseaArtFile(artPath, { root: tempDir, command: "build" });
    assert.ok(info);
    const scopeAttr = artScopeAttribute(artPath);
    assert.match(info.variants[0]?.template ?? "", new RegExp(`<div class="wrap" ${scopeAttr}>`));
    assert.match(info.variants[0]?.template ?? "", new RegExp(`<span ${scopeAttr}>`));
    assert.equal(info.styleBlocks?.[0], ".wrap { padding: 16px; }");
    assert.deepEqual(info.styleBlockDetails?.[0], {
      content: ".wrap { padding: 16px; }",
      lang: "css",
      scoped: true,
    });
  } finally {
    await fs.promises.rm(tempDir, { recursive: true, force: true });
  }
});

void test("processMuseaArtFile keeps arrow functions intact under a scoped style (#9752)", async () => {
  const tempDir = await fs.promises.mkdtemp(path.join(os.tmpdir(), "musea-scoped-arrow-"));
  const artPath = path.join(tempDir, "WithScopedStyle.art.vue");

  try {
    await fs.promises.writeFile(
      artPath,
      `<script setup lang="ts">
defineArt("./PressButton.vue", { title: "With scoped style" });
</script>

<style scoped>
button {
  padding: 8px;
}
</style>

<art title="A > B" action-events="press">
  <variant default name="Arrow">
    <PressButton @press="() => {}" />
  </variant>
</art>
`,
      "utf8",
    );

    const info = await processMuseaArtFile(artPath, { root: tempDir, command: "build" });
    assert.ok(info);
    const scopeAttr = artScopeAttribute(artPath);
    assert.equal(
      info.variants[0]?.template.trim(),
      `<PressButton @press="() => {}" ${scopeAttr} />`,
    );
    assert.deepEqual(info.metadata.actionEvents, ["press"]);
  } finally {
    await fs.promises.rm(tempDir, { recursive: true, force: true });
  }
});

void test("processMuseaArtFile keeps dev processing non-fatal", async () => {
  const tempDir = await fs.promises.mkdtemp(path.join(os.tmpdir(), "musea-dev-bad-art-"));
  const artPath = path.join(tempDir, "stories", "Broken.art.vue");
  const messages: string[] = [];

  try {
    await fs.promises.mkdir(path.dirname(artPath), { recursive: true });
    await fs.promises.writeFile(artPath, "<art>", "utf8");

    const info = await processMuseaArtFile(artPath, {
      root: tempDir,
      command: "serve",
      onError: (message) => messages.push(message),
    });

    assert.equal(info, null);
    assert.equal(messages.length, 1);
    assert.match(messages[0], /\[musea\] Failed to process stories\/Broken\.art\.vue/);
    assert.match(messages[0], /No <art> block found in file/);
  } finally {
    await fs.promises.rm(tempDir, { recursive: true, force: true });
  }
});

void test("reportArtStatusWarnings prints native unknown-status diagnostics", () => {
  const messages: string[] = [];
  reportArtStatusWarnings(
    {
      parseArtStatusWarnings: (source, options) => {
        assert.equal(source, '<art title="Button" status="wip"></art>');
        assert.equal(options?.filename, "button.art.vue");
        return [
          'button.art.vue: unknown status "wip"; falling back to "draft" (expected "draft" | "ready" | "deprecated")',
        ];
      },
    },
    '<art title="Button" status="wip"></art>',
    "button.art.vue",
    (message) => messages.push(message),
  );
  assert.deepEqual(messages, [
    '[musea] button.art.vue: unknown status "wip"; falling back to "draft" (expected "draft" | "ready" | "deprecated")',
  ]);
});

void test("reportArtStatusWarnings is a no-op when the native export is missing", () => {
  const messages: string[] = [];
  reportArtStatusWarnings({}, "<art></art>", "button.art.vue", (message) => messages.push(message));
  assert.deepEqual(messages, []);
});

void test("inline Art retains the native custom-block index after another block", async () => {
  const root = await fs.promises.mkdtemp(path.join(os.tmpdir(), "musea-inline-index-"));
  const file = path.join(root, "Card.vue");
  try {
    await fs.promises.writeFile(
      file,
      `<script setup>const label = '<art title="script text" />';</script>
<template><div>{{ label }}</div></template>
<docs>Owned by another plugin.</docs>
<art title="Registered" status="ready"><variant name="Default"><Self /></variant></art>`,
    );
    const art = await processMuseaArtFile(file, { root, command: "build" });
    assert.ok(art);
    assert.deepEqual(
      {
        isInline: art.isInline,
        componentPath: art.componentPath,
        inlineArtBlockIndex: art.inlineArtBlockIndex,
        metadata: art.metadata,
        variants: art.variants,
      },
      {
        isInline: true,
        componentPath: file,
        inlineArtBlockIndex: 1,
        metadata: {
          title: "Registered",
          description: undefined,
          component: undefined,
          category: undefined,
          tags: [],
          status: "ready",
          order: undefined,
          actionEvents: undefined,
        },
        variants: [{ name: "Default", template: "<Self />", isDefault: false, skipVrt: false }],
      },
    );
  } finally {
    await fs.promises.rm(root, { recursive: true, force: true });
  }
});
