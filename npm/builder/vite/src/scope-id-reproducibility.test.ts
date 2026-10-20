import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { build, type Rollup } from "vite";

import { vize } from "./index.ts";

const SFC = `<template><p class="note">hello</p></template>
<style scoped>.note { color: red; }</style>
`;

interface BuiltAsset {
  type: "css" | "js";
  fileName: string;
  sha256: string;
  scopeIds: string[];
}

function digest(body: string): string {
  return createHash("sha256").update(body).digest("hex");
}

async function buildCheckout(root: string): Promise<BuiltAsset[]> {
  fs.mkdirSync(path.join(root, "src"), { recursive: true });
  fs.writeFileSync(path.join(root, "entry.js"), 'export { default } from "./src/App.vue";\n');
  fs.writeFileSync(path.join(root, "src", "App.vue"), SFC);

  const result = (await build({
    root,
    configFile: false,
    logLevel: "silent",
    plugins: [vize({ configMode: false })],
    build: {
      write: false,
      minify: false,
      rollupOptions: {
        input: path.join(root, "entry.js"),
        external: [/^vue(?:\/|$)/],
      },
    },
  })) as Rollup.RollupOutput | Rollup.RollupOutput[];

  return (Array.isArray(result) ? result : [result])
    .flatMap((item) => item.output)
    .flatMap((item): BuiltAsset[] => {
      const type = item.type === "chunk" ? "js" : item.fileName.endsWith(".css") ? "css" : null;
      if (type === null) return [];
      const body = item.type === "chunk" ? item.code : String(item.source);
      return [
        {
          type,
          fileName: item.fileName,
          sha256: digest(body),
          scopeIds: [...new Set(body.match(/data-v-[0-9a-f]{8}/gu) ?? [])],
        },
      ];
    })
    .sort((left, right) => left.type.localeCompare(right.type));
}

void test("production CSS and JS asset hashes are stable across checkout directories", async () => {
  const parent = fs.mkdtempSync(path.join(os.tmpdir(), "vize-scope-checkouts-"));
  try {
    const first = await buildCheckout(path.join(parent, "checkout-a", "app"));
    const second = await buildCheckout(path.join(parent, "checkout-b", "somewhere", "else", "app"));

    assert.deepEqual(
      first.map((asset) => asset.type),
      ["css", "js"],
    );
    for (const asset of first) {
      assert.match(asset.fileName, /^assets\/entry-[A-Za-z0-9_-]+\.(?:css|js)$/u);
    }
    assert.ok(first.every((asset) => asset.scopeIds.length === 1));
    assert.deepEqual(first, second);
  } finally {
    fs.rmSync(parent, { recursive: true, force: true });
  }
});
