import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import type { ResolvedConfig } from "vite";

import type { ApiRoutesContext } from "./index.ts";
import { handleGenerate } from "./post-handlers.ts";

function stubContext(): ApiRoutesContext {
  return {
    config: { root: "/tmp/unused-generate-root" } as ResolvedConfig,
    artFiles: new Map(),
    scanRoots: [],
    tokensPath: undefined,
    basePath: "/__musea__",
    resolvedPreviewCss: [],
    resolvedPreviewSetup: null,
    devSessionToken: "test-session",
    processArtFile: async () => {},
    getDevServerPort: () => 5173,
  };
}

void test("handleGenerate rejects non-vue component paths before reading", async () => {
  let status = 200;
  let body = "";

  await handleGenerate(
    stubContext(),
    JSON.stringify({ componentPath: ".env" }),
    (data) => {
      body = JSON.stringify(data);
    },
    (message, code) => {
      status = code ?? 500;
      body = JSON.stringify({ error: message });
    },
  );

  assert.equal(status, 400);
  assert.deepEqual(JSON.parse(body), { error: "componentPath must be a .vue file" });
});

void test("handleGenerate rejects a .vue symlink whose realpath is not a Vue file", async () => {
  const tempDir = await fs.promises.mkdtemp(path.join(os.tmpdir(), "musea-generate-symlink-"));
  const root = path.join(tempDir, "root");

  try {
    await fs.promises.mkdir(root);
    const secret = path.join(root, ".env");
    await fs.promises.writeFile(secret, "SECRET=1\n");
    await fs.promises.symlink(secret, path.join(root, "Evil.vue"));

    let status = 200;
    let body = "";

    await handleGenerate(
      {
        ...stubContext(),
        config: { root } as ResolvedConfig,
      },
      JSON.stringify({ componentPath: "Evil.vue" }),
      (data) => {
        body = JSON.stringify(data);
      },
      (message, code) => {
        status = code ?? 500;
        body = JSON.stringify({ error: message });
      },
    );

    assert.equal(status, 400);
    assert.deepEqual(JSON.parse(body), { error: "componentPath must be a .vue file" });
  } finally {
    await fs.promises.rm(tempDir, { recursive: true, force: true });
  }
});
