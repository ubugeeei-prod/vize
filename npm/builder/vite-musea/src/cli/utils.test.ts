import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import { scanArtFiles } from "./utils.ts";

void test("scanArtFiles skips unreadable directories", async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-musea-scan-"));
  const locked = path.join(root, "locked");

  try {
    fs.mkdirSync(locked);
    fs.chmodSync(locked, 0);
    fs.writeFileSync(path.join(root, "Button.art.vue"), "<art></art>");

    const files = await scanArtFiles(root);

    assert.deepEqual(files, [path.join(root, "Button.art.vue")]);
  } finally {
    fs.chmodSync(locked, 0o700);
    fs.rmSync(root, { recursive: true, force: true });
  }
});
