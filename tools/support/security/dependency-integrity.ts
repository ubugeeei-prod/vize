import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import YAML from "yaml";

type DeepMerge = (
  target: Record<PropertyKey, unknown>,
  source: Record<PropertyKey, unknown>,
) => Record<PropertyKey, unknown>;

export const deepMergePatchPath = "patches/deepmerge@4.3.1.patch";
export const deepMergePatchHash =
  "f24df4b40d35b586a5fb81a5e778a8328ad0479e12d1828ea585dc2cce1965bd";
export const serializerVersion = "7.1.4";

function record(value: unknown): Record<string, unknown> {
  assert.ok(value && typeof value === "object" && !Array.isArray(value));
  return value as Record<string, unknown>;
}

function packageRequire(root: string, prefix: string, packageName: string) {
  const store = path.join(root, "node_modules/.pnpm");
  const entries = fs.readdirSync(store).filter((name) => name.startsWith(prefix));
  assert.equal(entries.length, 1, `${packageName} consumer must have one installed instance`);
  return createRequire(path.join(store, entries[0], "node_modules", packageName, "package.json"));
}

function assertDependencyReferences(
  snapshots: Record<string, unknown>,
  packageName: string,
  expectedReference: string,
): void {
  let references = 0;
  for (const snapshotValue of Object.values(snapshots)) {
    const snapshot = record(snapshotValue);
    for (const field of ["dependencies", "optionalDependencies"]) {
      const dependencies = record(snapshot[field] ?? {});
      if (!Object.hasOwn(dependencies, packageName)) continue;
      assert.equal(dependencies[packageName], expectedReference);
      references += 1;
    }
  }
  assert.ok(references > 0, `${packageName} must remain reachable`);
}

function assertMergeRuntime(root: string): number {
  const require = packageRequire(
    root,
    "@rollup+plugin-node-resolve@16.0.3_",
    "@rollup/plugin-node-resolve",
  );
  const packageJson = require.resolve("deepmerge/package.json");
  assert.match(packageJson, new RegExp(`deepmerge@4\\.3\\.1_patch_hash=${deepMergePatchHash}`));
  const merge = require("deepmerge") as DeepMerge;
  const source = JSON.parse(
    '{"__proto__":{"isAdmin":true},"constructor":{"isAdmin":true},"prototype":{"isAdmin":true},"safe":{"right":2}}',
  ) as Record<PropertyKey, unknown>;
  const targets = [
    Object.create(null),
    { __proto__: null },
    Object.defineProperty({}, "__proto__", { value: null, enumerable: true, writable: true }),
  ] as Record<PropertyKey, unknown>[];
  let checks = 0;

  for (const target of targets) {
    const merged = merge(target, source);
    assert.equal(merged.isAdmin, undefined);
    assert.equal(Object.getPrototypeOf(merged), Object.prototype);
    assert.equal(Object.hasOwn(merged, "__proto__"), false);
    assert.equal(Object.hasOwn(merged, "constructor"), false);
    assert.equal(Object.hasOwn(merged, "prototype"), false);
    assert.deepEqual(merged.safe, { right: 2 });
    checks += 6;
  }

  assert.deepEqual(merge({ safe: { left: 1 } }, { safe: { right: 2 } }), {
    safe: { left: 1, right: 2 },
  });
  checks += 1;
  const hostileTarget = new Proxy(
    {},
    {
      has() {
        throw new Error("untrusted target");
      },
    },
  );
  assert.deepEqual(merge(hostileTarget, { safe: true }), {});
  return checks + 1;
}

export function verifyDependencyIntegrity(root: string): {
  deepMergePatch: string;
  mergeChecks: number;
  serializer: string;
} {
  const workspace = record(
    YAML.parse(fs.readFileSync(path.join(root, "pnpm-workspace.yaml"), "utf8")),
  );
  const documents = YAML.parseAllDocuments(
    fs.readFileSync(path.join(root, "pnpm-lock.yaml"), "utf8"),
  );
  assert.equal(documents.length, 2);
  assert.ok(documents.every((document) => document.errors.length === 0));
  const lock = record(documents[1].toJS());
  const workspacePatches = record(workspace.patchedDependencies);
  const lockPatches = record(lock.patchedDependencies);
  const overrides = record(workspace.overrides);
  const packages = record(lock.packages);
  const snapshots = record(lock.snapshots);

  assert.equal(workspacePatches["deepmerge@4.3.1"], deepMergePatchPath);
  assert.equal(lockPatches["deepmerge@4.3.1"], deepMergePatchHash);
  assert.equal(
    createHash("sha256")
      .update(fs.readFileSync(path.join(root, deepMergePatchPath)))
      .digest("hex"),
    deepMergePatchHash,
  );
  assert.deepEqual(
    Object.keys(snapshots).filter((key) => key.startsWith("deepmerge@")),
    [`deepmerge@4.3.1(patch_hash=${deepMergePatchHash})`],
  );
  assertDependencyReferences(snapshots, "deepmerge", `4.3.1(patch_hash=${deepMergePatchHash})`);

  assert.equal(overrides["serialize-javascript@>=7.0.0 <7.1.3"], serializerVersion);
  assert.ok(packages[`serialize-javascript@${serializerVersion}`]);
  assert.equal(packages["serialize-javascript@7.1.0"], undefined);
  assertDependencyReferences(snapshots, "serialize-javascript", serializerVersion);
  const serializerRequire = packageRequire(
    root,
    "@rollup+plugin-terser@1.0.0_",
    "@rollup/plugin-terser",
  );
  const serializerPackage = record(
    JSON.parse(
      fs.readFileSync(serializerRequire.resolve("serialize-javascript/package.json"), "utf8"),
    ),
  );
  assert.equal(serializerPackage.version, serializerVersion);

  return {
    deepMergePatch: deepMergePatchHash,
    mergeChecks: assertMergeRuntime(root),
    serializer: serializerVersion,
  };
}
