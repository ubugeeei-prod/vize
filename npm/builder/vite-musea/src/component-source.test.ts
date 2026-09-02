import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { resolveComponentSourcePath } from "./component-source.ts";
import { HttpError } from "./http-error.ts";
import type { ArtFileInfo } from "./types/index.ts";

function createArt(pathname: string, component: string): ArtFileInfo {
  return {
    path: pathname,
    metadata: { title: "Card", component, tags: [], status: "ready" },
    variants: [],
    hasScriptSetup: false,
    hasScript: false,
    styleCount: 0,
  };
}

void test("resolveComponentSourcePath rejects a .vue symlink whose realpath is not a Vue file", async () => {
  const tempDir = await fs.promises.mkdtemp(path.join(os.tmpdir(), "musea-component-source-"));
  const artPath = path.join(tempDir, "Card.art.vue");
  const decoy = path.join(tempDir, "Evil.vue");
  const secret = path.join(tempDir, ".env");

  try {
    await fs.promises.writeFile(secret, "SECRET=1\n");
    await fs.promises.symlink(secret, decoy);
    await fs.promises.writeFile(artPath, "");

    assert.throws(
      () => resolveComponentSourcePath(createArt(artPath, "./Evil.vue"), artPath, [tempDir]),
      (error: unknown) =>
        error instanceof HttpError &&
        error.status === 400 &&
        error.message === "component path must be a .vue file",
    );
  } finally {
    await fs.promises.rm(tempDir, { recursive: true, force: true });
  }
});
