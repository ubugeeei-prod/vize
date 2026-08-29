import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import path from "node:path";

import { test } from "vite-plus/test";

test("package check keeps the UI source corpus on the toolchain gate", async () => {
  const manifest = JSON.parse(await readFile(path.resolve("package.json"), "utf8")) as {
    readonly scripts: Readonly<Record<string, string>>;
  };

  assert.equal(
    manifest.scripts["lint:sfc"],
    "vp exec node scripts/lint-sfc.ts src && vp exec node scripts/check-renderers.ts src",
  );
  assert.match(manifest.scripts.check, /^pnpm lint:sfc && /);
  assert.match(manifest.scripts.check, /vue-tsc --noEmit -p tsconfig\.typecheck\.json/);
});

test("renderer conformance script owns the DOM, SSR, and Vapor lanes", async () => {
  const checkRendererSource = await readFile(path.resolve("scripts/check-renderers.ts"), "utf8");

  assert.ok(checkRendererSource.includes('name: "dom"'));
  assert.ok(checkRendererSource.includes('name: "ssr"'));
  assert.ok(checkRendererSource.includes('name: "vapor"'));
  assert.ok(checkRendererSource.includes("compileSfc"));
  assert.ok(checkRendererSource.includes("sourceFiles.length + inlineFixtures.length"));
});
