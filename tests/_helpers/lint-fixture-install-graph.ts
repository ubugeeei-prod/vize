import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

import {
  assertPinnedFixtureFiles,
  sha256,
  type PinnedFixtureSource,
} from "./pinned-fixture-source.ts";

/** A migrated dependency graph never changes the files consumed by lint. */
export function prepareSeparateLintInstallGraph(
  sourceRoot: string,
  installRoot: string,
  source: PinnedFixtureSource,
  closureDir: string,
) {
  assert.notEqual(fs.realpathSync(sourceRoot), fs.realpathSync(installRoot));
  assertPinnedFixtureFiles(sourceRoot, source);
  assertPinnedFixtureFiles(installRoot, source);
  const metadata = JSON.parse(fs.readFileSync(path.join(closureDir, "source.json"), "utf8"));
  assert.equal(metadata.revision, source.revision, "migration must match the authored source pin");
  assert.equal(metadata.originalFrozenClosure, false, "a migrated closure is explicitly distinct");
  for (const [relativePath, expected] of Object.entries(metadata.inputs)) {
    assert.equal(sha256(fs.readFileSync(path.join(sourceRoot, relativePath))), expected);
  }
  const originalPackage = fs.readFileSync(path.join(sourceRoot, "package.json"), "utf8");
  const effectivePackage = fs.readFileSync(path.join(closureDir, "package.json"), "utf8");
  const { originalDependency, replacement } = metadata;
  assert.equal(replacement.name, originalDependency.name);
  assert.equal(
    JSON.parse(originalPackage).dependencies[originalDependency.name],
    originalDependency.specifier,
  );
  assert.equal(
    effectivePackage,
    originalPackage.replace(
      JSON.stringify(originalDependency.specifier),
      JSON.stringify(replacement.specifier),
    ),
    "only the explicitly reviewed dependency declaration may change in the installer manifest",
  );
  const effectiveBlobs = source.blobs.map((blob) => {
    if (!(blob.path in metadata.effectiveInputs)) return blob;
    assert.ok(
      ["package.json", "pnpm-lock.yaml"].includes(blob.path),
      "only installer manifests may change",
    );
    const bytes = fs.readFileSync(path.join(closureDir, blob.path));
    assert.equal(
      sha256(bytes),
      metadata.effectiveInputs[blob.path],
      `effective input changed: ${blob.path}`,
    );
    fs.writeFileSync(path.join(installRoot, blob.path), bytes);
    const object = createHash("sha1").update(`blob ${bytes.length}\0`).update(bytes).digest("hex");
    return { ...blob, object };
  });
  assert.deepEqual(Object.keys(metadata.effectiveInputs).sort(), [
    "package.json",
    "pnpm-lock.yaml",
  ]);
  const effective = {
    ...source,
    blobs: effectiveBlobs,
    inventorySha256: sha256(JSON.stringify(effectiveBlobs)),
  };
  let attached = false;
  const verify = () => {
    assertPinnedFixtureFiles(sourceRoot, source);
    assertPinnedFixtureFiles(installRoot, effective);
    if (attached) {
      assert.equal(
        fs.realpathSync(path.join(sourceRoot, "node_modules")),
        fs.realpathSync(path.join(installRoot, "node_modules")),
        "lint must retain its explicitly reviewed dependency graph",
      );
    }
  };
  verify();
  return {
    verify,
    receipt: { ...metadata, effectiveInventorySha256: effective.inventorySha256 },
    attach() {
      verify();
      const modules = path.join(installRoot, "node_modules");
      assert.ok(
        fs.statSync(modules).isDirectory(),
        "effective dependencies must be installed first",
      );
      const link = path.join(sourceRoot, "node_modules");
      assert.ok(!source.blobs.some(({ path: original }) => original.startsWith("node_modules/")));
      fs.rmSync(link, { recursive: true, force: true });
      fs.symlinkSync(modules, link, process.platform === "win32" ? "junction" : "dir");
      attached = true;
      verify();
    },
  };
}
