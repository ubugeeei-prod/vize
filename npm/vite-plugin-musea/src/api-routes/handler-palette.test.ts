import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import type { IncomingMessage, ServerResponse } from "node:http";
import { Readable } from "node:stream";
import os from "node:os";
import path from "node:path";
import type { ResolvedConfig } from "vite";

import { createApiMiddleware, type ApiRoutesContext } from "./index.ts";
import type { ArtFileInfo } from "../types/art.ts";

function createArt(pathname: string): ArtFileInfo {
  return {
    path: pathname,
    metadata: { title: "Inline Input", tags: [], status: "ready" },
    variants: [],
    hasScriptSetup: false,
    hasScript: false,
    styleCount: 0,
    isInline: true,
    componentPath: pathname,
  };
}

function createContext(root: string, artFiles: Map<string, ArtFileInfo>): ApiRoutesContext {
  return {
    config: { root } as ResolvedConfig,
    artFiles,
    scanRoots: [root],
    tokensPath: undefined,
    basePath: "/__musea__",
    resolvedPreviewCss: [],
    resolvedPreviewSetup: null,
    devSessionToken: "test-session",
    processArtFile: async () => {},
    getDevServerPort: () => 5173,
  };
}

async function invokeApi(ctx: ApiRoutesContext, url: string): Promise<string> {
  return await new Promise((resolve, reject) => {
    const req = Readable.from([]) as IncomingMessage;
    req.method = "GET";
    req.url = url;
    req.headers = {};

    const res = {
      statusCode: 200,
      setHeader() {},
      end(chunk?: Buffer | string) {
        resolve(Buffer.isBuffer(chunk) ? chunk.toString("utf-8") : String(chunk ?? ""));
      },
    } as ServerResponse;

    Promise.resolve(createApiMiddleware(ctx)(req, res, () => resolve(""))).catch(reject);
  });
}

void test("palette merges variant values with component prop metadata", async () => {
  const tempDir = await fs.promises.mkdtemp(path.join(os.tmpdir(), "musea-palette-merge-"));
  const componentPath = path.join(tempDir, "src", "InlineInput.vue");

  try {
    await fs.promises.mkdir(path.dirname(componentPath), { recursive: true });
    await fs.promises.writeFile(
      componentPath,
      `<script setup lang="ts">
defineProps<{
  modelValue?: string;
  tone?: "brand" | "neutral" | "danger";
  count?: number;
  disabled?: boolean;
}>()
</script>

<template>
  <input :value="modelValue" :disabled="disabled" :data-tone="tone" :data-count="count" />
</template>

<art title="Inline Input">
  <variant name="Default">
    <Self model-value="Seed" tone="brand" :disabled="true" />
  </variant>
</art>
`,
      "utf-8",
    );

    const art = createArt(componentPath);
    const ctx = createContext(tempDir, new Map([[componentPath, art]]));
    const body = await invokeApi(ctx, `/arts/${encodeURIComponent(componentPath)}/palette`);
    const controls = JSON.parse(body).controls as Array<{
      name: string;
      control: string;
      default_value?: unknown;
      options: Array<{ label: string; value: unknown }>;
    }>;
    const byName = new Map(controls.map((control) => [control.name, control]));

    assert.equal(byName.has("model-value"), false);
    assert.equal(byName.get("modelValue")?.control, "text");
    assert.equal(byName.get("tone")?.control, "select");
    assert.deepEqual(byName.get("tone")?.options, [
      { label: "brand", value: "brand" },
      { label: "neutral", value: "neutral" },
      { label: "danger", value: "danger" },
    ]);
    assert.equal(byName.get("count")?.control, "number");
    assert.equal(byName.get("disabled")?.control, "boolean");
  } finally {
    await fs.promises.rm(tempDir, { recursive: true, force: true });
  }
});
