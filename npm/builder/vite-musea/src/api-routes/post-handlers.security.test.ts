import assert from "node:assert/strict";
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
