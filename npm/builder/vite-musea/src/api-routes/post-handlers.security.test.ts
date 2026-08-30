import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import type { ResolvedConfig } from "vite";

import type { ApiRoutesContext } from "./index.ts";
import { handleGenerate } from "./post-handlers.ts";

function stubContext(root: string): ApiRoutesContext {
  return {
    config: { root } as ResolvedConfig,
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

async function generate(
  root: string,
  componentPath: string,
): Promise<{ status: number; body: string }> {
  let status = 200;
  let body = "";

  await handleGenerate(
    stubContext(root),
    JSON.stringify({ componentPath }),
    (data) => {
      body = JSON.stringify(data);
    },
    (message, code) => {
      status = code ?? 500;
      body = JSON.stringify({ error: message });
    },
  );

  return { status, body };
}

void test("handleGenerate rejects non-vue component paths before reading", async () => {
  const { status, body } = await generate("/tmp/unused-generate-root", ".env");

  assert.equal(status, 400);
  assert.deepEqual(JSON.parse(body), { error: "componentPath must be a .vue file" });
});

void test("handleGenerate rejects a .vue symlink that targets a non-vue file", async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "musea-generate-link-"));
  try {
    fs.writeFileSync(path.join(root, ".env"), "SECRET=1\n");
    fs.symlinkSync(path.join(root, ".env"), path.join(root, "Evil.vue"));

    const { status, body } = await generate(root, "Evil.vue");

    assert.equal(status, 400);
    assert.deepEqual(JSON.parse(body), { error: "componentPath must be a .vue file" });
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});
