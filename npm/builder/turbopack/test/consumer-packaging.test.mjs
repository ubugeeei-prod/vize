import assert from "node:assert/strict";
import { test } from "node:test";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { assertBundledHelpers } from "./prepare-consumer.mjs";
import { sourceNativeResolutions } from "./consumer-lock.mjs";

void test("public bundle inspection visits nested loader entries and rejects private imports", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "vize-turbopack-packaging-"));
  try {
    await fs.mkdir(path.join(root, "loader"));
    await fs.writeFile(
      path.join(root, "index.mjs"),
      'export { default } from "./loader/index.mjs";',
    );
    const entry = path.join(root, "loader/index.mjs");
    await fs.writeFile(entry, "export default function loader() {};");
    await assertBundledHelpers(root);
    for (const statement of [
      'import { MappedModule } from "@vizejs/builder-shared/mapped-module";',
      'const helper = require("@vizejs/builder-shared/mapped-module");',
      'const helper = import("@vizejs/builder-shared/mapped-module");',
    ]) {
      await fs.writeFile(entry, statement);
      await assert.rejects(assertBundledHelpers(root));
    }
  } finally {
    await fs.rm(root, { recursive: true, force: true });
  }
});

void test("source custody reads pnpm project documents and rejects alternate native resolutions", () => {
  const project = {
    importers: {
      ".": {
        dependencies: { "@vizejs/turbopack-plugin": { version: "file:../packs/turbopack.tgz" } },
      },
    },
    overrides: { "@vizejs/native": "file:../packs/native.tgz" },
    packages: {
      "@vizejs/native@file:../packs/native.tgz": {
        resolution: { tarball: "file:../packs/native.tgz", integrity: "sha512-selected" },
      },
    },
  };
  const manager = "---\nlockfileVersion: '9.0'\npackages: {}\n";
  const serialize = (value) => "---\n" + JSON.stringify(value) + "\n";
  const check = (text) =>
    sourceNativeResolutions(
      text,
      "/receipt/consumer",
      "/receipt/packs/native.tgz",
      "sha512-selected",
    );
  assert.equal(check(serialize(project)).length, 1);
  assert.equal(check(manager + serialize(project)).length, 1);
  const missingOverride = structuredClone(project);
  delete missingOverride.overrides;
  const registry = structuredClone(project);
  registry.packages["@vizejs/native@0.474.0"] = Object.values(registry.packages)[0];
  const wrongIntegrity = structuredClone(project);
  Object.values(wrongIntegrity.packages)[0].resolution.integrity = "sha512-other";
  const otherFile = structuredClone(project);
  Object.values(otherFile.packages)[0].resolution.tarball = "file:../packs/other.tgz";
  for (const invalid of [missingOverride, registry, wrongIntegrity, otherFile]) {
    assert.throws(() => check(manager + serialize(invalid)));
  }
  assert.throws(() => check(serialize({ packages: registry.packages }) + serialize(project)));
  assert.throws(() => check(serialize(project) + serialize(project)));
  assert.throws(() => check("{ invalid YAML\n"));
});
