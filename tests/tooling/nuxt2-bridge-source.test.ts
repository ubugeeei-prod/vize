import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { prepareNuxt2BridgeSource } from "../../tools/support/compat/nuxt/nuxt2-bridge-source.ts";

function fixture() {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-bridge-source-"));
  const root = path.join(directory, "source");
  const host = path.join(directory, "host");
  const artifacts = path.join(directory, "artifacts");
  const sources = {
    "@vizejs/nuxt": "npm/framework/nuxt",
    "@vizejs/nuxt-lint-config": "npm/framework/nuxt-lint-config",
    "@vizejs/native": "npm/native",
    "oxlint-plugin-vize": "npm/oxlint",
    vize: "npm/cli",
  };
  const writePackage = (target: string, name: string, version = "0.0.0") => {
    fs.mkdirSync(target, { recursive: true });
    fs.writeFileSync(path.join(target, "package.json"), JSON.stringify({ name, version }));
  };
  for (const [name, relative] of Object.entries(sources)) {
    const source = path.join(root, relative);
    writePackage(source, name);
    fs.mkdirSync(path.join(source, "dist"));
    fs.writeFileSync(path.join(source, "dist/index.mjs"), "export default {};\n");
  }
  writePackage(path.join(root, "npm/framework/nuxt/node_modules/@nuxt/kit"), "@nuxt/kit", "3.11.2");
  writePackage(path.join(host, "node_modules/@nuxt/bridge"), "@nuxt/bridge", "3.6.1");
  writePackage(
    path.join(host, "node_modules/@nuxt/bridge/node_modules/@nuxt/kit"),
    "@nuxt/kit",
    "3.18.1",
  );
  fs.mkdirSync(artifacts);
  return {
    root,
    host,
    artifacts,
    targets: Object.keys(sources).map((name) => path.join(host, "node_modules", name)),
    cleanup: () => fs.rmSync(directory, { recursive: true, force: true }),
  };
}

void test("partial source preparation rolls back owned targets and supports a clean retry", () => {
  const f = fixture();
  try {
    const missing = path.join(f.root, "npm/framework/nuxt-lint-config/dist");
    fs.rmSync(missing, { recursive: true });
    assert.throws(() => prepareNuxt2BridgeSource(f.root, f.host, f.artifacts), /ENOENT/);
    for (const target of f.targets) assert.equal(fs.existsSync(target), false, target);
    fs.mkdirSync(missing);
    fs.writeFileSync(path.join(missing, "index.mjs"), "export default {};\n");
    const cleanup = prepareNuxt2BridgeSource(f.root, f.host, f.artifacts);
    for (const target of f.targets) assert.equal(fs.existsSync(target), true, target);
    const packet = JSON.parse(
      fs.readFileSync(path.join(f.artifacts, "bridge-source-products.json"), "utf8"),
    );
    assert.deepEqual(
      packet.kits.map(({ role, version }: { role: string; version: string }) => ({
        role,
        version,
      })),
      [
        { role: "vize-module", version: "3.11.2" },
        { role: "bridge-host", version: "3.18.1" },
      ],
    );
    cleanup();
    cleanup();
    for (const target of f.targets) assert.equal(fs.existsSync(target), false, target);
    assert.ok(
      fs.existsSync(path.join(f.root, "npm/native/package.json")),
      "source products remain intact",
    );
  } finally {
    f.cleanup();
  }
});

void test("a pre-existing host package is refused and retained while earlier owned targets roll back", () => {
  const f = fixture();
  try {
    const existing = path.join(f.host, "node_modules/oxlint-plugin-vize");
    fs.mkdirSync(existing);
    const marker = path.join(existing, "host-owned.txt");
    fs.writeFileSync(marker, "original host bytes\n");
    assert.throws(
      () => prepareNuxt2BridgeSource(f.root, f.host, f.artifacts),
      /explicit source product/,
    );
    assert.equal(fs.readFileSync(marker, "utf8"), "original host bytes\n");
    for (const target of f.targets.filter((target) => target !== existing))
      assert.equal(fs.existsSync(target), false, target);
  } finally {
    f.cleanup();
  }
});
