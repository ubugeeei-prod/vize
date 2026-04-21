import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

import { resolveScanRoots, scanArtFiles } from "./utils.ts";

void test("resolveScanRoots preserves include bases outside the Vite root", () => {
  const root = "/workspace/apps/website";
  const roots = resolveScanRoots(root, ["../../packages/ui/src/**/*.art.vue"]);

  assert.deepEqual(roots, ["/workspace/packages/ui/src"]);
});

void test("scanArtFiles discovers art files outside the Vite root when include points upward", async () => {
  const tempDir = await fs.promises.mkdtemp(path.join(os.tmpdir(), "musea-scan-"));
  const root = path.join(tempDir, "apps", "website");
  const externalDir = path.join(tempDir, "packages", "ui", "src");
  const artFile = path.join(externalDir, "MfButton.art.vue");

  await fs.promises.mkdir(root, { recursive: true });
  await fs.promises.mkdir(externalDir, { recursive: true });
  await fs.promises.writeFile(artFile, "<art><template><div /></template></art>\n", "utf-8");

  const files = await scanArtFiles(root, ["../../packages/ui/src/**/*.art.vue"], [], false);

  assert.deepEqual(files, [artFile]);

  await fs.promises.rm(tempDir, { recursive: true, force: true });
});
