import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

import { dualLspProviderRoot } from "./dual-lsp-product.ts";

const expectedDependencies = {
  "@vue/language-server": "3.3.13",
  "@vue/typescript-plugin": "3.3.13",
  typescript: "6.0.3",
  "typescript-language-server": "6.0.2",
  vue: "3.6.0-rc.11",
};

function sha256(value: string): string {
  return createHash("sha256").update(value).digest("hex");
}

export function dualLspProviderDependencyReceipt() {
  const packageJsonPath = path.join(dualLspProviderRoot, "package.json");
  const lockfilePath = path.join(dualLspProviderRoot, "pnpm-lock.yaml");
  const packageJson = fs.readFileSync(packageJsonPath, "utf8");
  const lockfile = fs.readFileSync(lockfilePath, "utf8");
  const manifest = JSON.parse(packageJson) as { dependencies?: unknown };
  assert.deepEqual(
    manifest.dependencies,
    expectedDependencies,
    "isolated provider dependency pins drifted",
  );
  for (const entry of [
    "'@vue/language-server@3.3.13'",
    "'@vue/typescript-plugin@3.3.13'",
    "typescript-language-server@6.0.2",
    "typescript@6.0.3",
  ]) {
    assert.ok(lockfile.includes(entry), `isolated provider lock omitted ${entry}`);
  }
  const payload = {
    providerRoot: dualLspProviderRoot,
    dependencies: expectedDependencies,
    packageJson: { path: packageJsonPath, sha256: sha256(packageJson) },
    lockfile: { path: lockfilePath, sha256: sha256(lockfile) },
  };
  return { ...payload, sha256: sha256(JSON.stringify(payload)) };
}
