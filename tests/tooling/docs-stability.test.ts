import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

test("stability page documents v1 alpha support tiers", () => {
  const stability = fs.readFileSync(path.join(root, "docs/content/stability.md"), "utf8");

  assert.match(stability, /# Stability/);
  assert.match(stability, /v1 alpha/);
  assert.match(stability, /Node 22/);
  assert.match(stability, /`oxlint-plugin-vize`[\s\S]*`\^22 \|\| >= 24`/);
  assert.match(stability, /linux-arm64-gnu/);
  assert.match(stability, /win32-arm64-msvc/);
  assert.match(stability, /linux-x64-musl/);
  assert.match(stability, /linux-arm64-musl/);
  assert.match(stability, /`vize --version`/);
  assert.match(stability, /`vize check`/);
  assert.match(stability, /`@vizejs\/native` through both `require` and `import`/);

  for (const tier of ["Alpha-supported", "Compatibility preview", "Experimental", "Incubating"]) {
    assert.match(stability, new RegExp(`\\| ${tier}\\s+\\|`));
  }

  for (const packageName of [
    "vize",
    "@vizejs/native",
    "@vizejs/vite-plugin",
    "@vizejs/unplugin",
    "@vizejs/rspack-plugin",
    "@vizejs/nuxt",
    "@vizejs/vite-plugin-musea",
    "@vizejs/wasm",
    "@vizejs/fresco",
  ]) {
    assert.match(stability, new RegExp(escapeRegExp(`\`${packageName}\``)));
  }
});

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
