import assert from "node:assert/strict";
import { dirname, resolve } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { build } from "vite-plus";

const packageRoot = fileURLToPath(new URL("..", import.meta.url));
const sourceDirectory = fileURLToPath(new URL(".", import.meta.url));
const adapterEntry = fileURLToPath(new URL("adapter.ts", import.meta.url));
const VIRTUAL_ID = "virtual:marquette-adapter-treeshake";
const RESOLVED_ID = `\0${VIRTUAL_ID}`;

async function bundle(entryCode: string): Promise<string> {
  const result = await build({
    configFile: false,
    logLevel: "error",
    root: packageRoot,
    plugins: [
      {
        name: "marquette-adapter-treeshake",
        enforce: "pre",
        resolveId(id: string, importer: string | undefined) {
          if (id === VIRTUAL_ID) return RESOLVED_ID;
          const importedFrom = importer === RESOLVED_ID ? packageRoot : dirname(importer ?? "/");
          const target = id.startsWith(".") ? resolve(importedFrom, id) : id;
          if (target.startsWith(sourceDirectory) && target.endsWith(".ts")) {
            return { id: target, moduleSideEffects: true };
          }
          return undefined;
        },
        load(id: string) {
          return id === RESOLVED_ID ? entryCode : undefined;
        },
      },
    ],
    build: {
      write: false,
      minify: false,
      target: "es2022",
      reportCompressedSize: false,
      rollupOptions: {
        input: VIRTUAL_ID,
        preserveEntrySignatures: "strict",
        output: { format: "es" },
      },
    },
  });
  assert.ok(!Array.isArray(result));
  assert.ok("output" in result);
  return result.output.map((item) => (item.type === "chunk" ? item.code : "")).join("\n");
}

void test("the adapter entry has no retained module-level side effects", async () => {
  const code = await bundle(`import ${JSON.stringify(adapterEntry)};`);
  assert.equal(code.replaceAll(/\/\*[^]*?\*\//g, "").trim(), "");
});

void test("negotiation excludes compatibility-only implementation", async () => {
  const code = await bundle(
    `export { negotiateAdapterCapabilities } from ${JSON.stringify(adapterEntry)};`,
  );
  assert.match(code, /missing-capability/);
  assert.doesNotMatch(code, /capability support was added/);
  assert.doesNotMatch(code, /minimum supported version decreased/);
});

void test("compatibility excludes negotiation-only implementation", async () => {
  const code = await bundle(
    `export { compareAdapterCapabilities } from ${JSON.stringify(adapterEntry)};`,
  );
  assert.match(code, /capability support was added/);
  assert.doesNotMatch(code, /missing-capability/);
  assert.doesNotMatch(code, /unknown-requirement/);
});
