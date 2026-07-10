import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import { processMuseaArtFile } from "./art-processing.js";

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
