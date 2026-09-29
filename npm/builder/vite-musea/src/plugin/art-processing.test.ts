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
    assert.match(info.styleBlocks?.[0] ?? "", new RegExp(`\\.${"wrap"}\\[${scopeAttr}\\]`));
    assert.doesNotMatch(info.styleBlocks?.[0] ?? "", /^\.wrap \{/);
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
