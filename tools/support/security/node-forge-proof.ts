import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import YAML from "yaml";

import { verifyForgeSecurity } from "./node-forge-laws.ts";
import { assertNoPackageConsumers, protectedModule } from "./node-forge-consumers.ts";

export const advisoryId = "GHSA-86w9-cpqp-85rv";
export const patchPath = "patches/node-forge@1.4.0.patch";
export const patchHash = "8877267668e88fba4baeb2b574509e2d13a9cff4170179473cb808be833d2b1b";
export const rsaHash = "bb2c61cef273c89aaedb9c865173f5330d76917024ff1c2c1d39ffd5be01cc2d";
const forgeHash = "da8f76d490bd09144144e40a7cfea386b36750bf5d500f8b7eb6785087805aaa";
const listhenHash = "8db9468e5c0b6e06a535f273a04c14006cfe673a12f409e263b93236e0a1c534";
const forgeIntegrity =
  "sha512-LarFH0+6VfriEhqMMcLX2F7SwSXeWwnEAJEsYm5QKWchiVYVvJyV9v7UDvUv+w5HO23ZpQTXDv/GxdDdMyOuoQ==";
const listhenIntegrity =
  "sha512-6nt/86SkqUQSLW1ofz8MxC6RhRMqOl3ONISe6qqvJ3xj09aJWQx6DhgSZpugs3PX4PXdOas/WD6A9jx6J2N19A==";
const forgeRef = "1.4.0(patch_hash=" + patchHash + ")";
const listhenRef = "1.10.1(@parcel/watcher@2.5.6)(srvx@0.11.15)";

function record(value: unknown): Record<string, unknown> {
  assert.ok(value && typeof value === "object" && !Array.isArray(value), "expected record");
  return value as Record<string, unknown>;
}

export function assertForgeLock(workspaceValue: unknown, lockValue: unknown): void {
  const workspace = record(workspaceValue);
  assert.ok(!workspace.audit, "workspace audit suppression is outside this proof");
  assert.equal(record(workspace.overrides)["node-forge"], "1.4.0");
  assert.equal(record(workspace.patchedDependencies)["node-forge@1.4.0"], patchPath);
  const lock = record(lockValue);
  assert.equal(record(lock.overrides)["node-forge"], "1.4.0");
  assert.deepEqual(record(lock.patchedDependencies)["node-forge@1.4.0"], {
    hash: patchHash,
    path: patchPath,
  });
  const packages = record(lock.packages);
  const forgeKeys = Object.keys(packages).filter((key) => key.startsWith("node-forge@"));
  assert.deepEqual(forgeKeys, ["node-forge@1.4.0"]);
  assert.deepEqual(record(packages[forgeKeys[0]]).resolution, { integrity: forgeIntegrity });
  assert.deepEqual(record(packages["listhen@1.10.1"]).resolution, { integrity: listhenIntegrity });
  const snapshots = record(lock.snapshots);
  assert.deepEqual(
    Object.keys(snapshots).filter((key) => key.startsWith("node-forge@")),
    ["node-forge@" + forgeRef],
  );
  const forgeEdges: string[] = [];
  const listhenParents: string[] = [];
  for (const [owner, snapshotValue] of Object.entries(snapshots)) {
    const snapshot = record(snapshotValue);
    for (const field of ["dependencies", "optionalDependencies"]) {
      for (const [name, ref] of Object.entries(record(snapshot[field] ?? {}))) {
        assert.equal(typeof ref, "string");
        if (name === "node-forge" || /(?:^|:)node-forge@/.test(String(ref))) {
          assert.equal(name, "node-forge", "new aliased Forge consumer");
          assert.equal(ref, forgeRef, "unpatched Forge edge");
          forgeEdges.push(owner);
        }
        if (name === "listhen" || /(?:^|:)listhen@/.test(String(ref))) {
          assert.equal(name, "listhen", "new aliased listhen consumer");
          assert.equal(ref, listhenRef);
          listhenParents.push(owner);
        }
      }
    }
  }
  assert.deepEqual(forgeEdges, ["listhen@" + listhenRef]);
  assert.equal(listhenParents.length, 2);
  assert.ok(listhenParents.some((key) => key.startsWith("@nuxt/cli@3.37.0(")));
  assert.ok(listhenParents.some((key) => key.startsWith("nitropack@2.13.4(")));
  for (const importerValue of Object.values(record(lock.importers))) {
    for (const field of ["dependencies", "devDependencies", "optionalDependencies"]) {
      for (const [name, value] of Object.entries(record(record(importerValue)[field] ?? {}))) {
        const dependency = record(value);
        assert.ok(
          ![name, dependency.specifier, dependency.version].some((value) =>
            protectedModule(String(value)),
          ),
          "direct or aliased Forge/listhen consumer",
        );
      }
    }
  }
}

function auditEntries(value: unknown): {
  actionable: Record<string, unknown>[];
  counts: Record<string, unknown>;
} {
  const report = record(value);
  assert.deepEqual(Object.keys(report).sort(), ["advisories", "metadata"]);
  const levels = ["info", "low", "moderate", "high", "critical"];
  const counts = record(record(report.metadata).vulnerabilities);
  const printed = Object.fromEntries(levels.map((level) => [level, 0]));
  const actionable: Record<string, unknown>[] = [];
  for (const entry of Object.values(record(report.advisories))) {
    const advisory = record(entry);
    assert.ok(
      typeof advisory.severity === "string" && levels.includes(advisory.severity),
      "unknown severity",
    );
    printed[advisory.severity]++;
    if (levels.indexOf(advisory.severity) >= 2) actionable.push(advisory);
  }
  for (const level of levels) {
    assert.ok(Number.isSafeInteger(counts[level]) && Number(counts[level]) >= 0);
    if (levels.indexOf(level) >= 2)
      assert.equal(printed[level], counts[level], "missing actionable finding");
    else assert.ok(printed[level] <= Number(counts[level]), "inconsistent below-threshold count");
  }
  return { actionable, counts };
}

export function assertRemediatedReport(value: unknown): void {
  const { actionable, counts } = auditEntries(value);
  assert.equal(actionable.length, 1, "unexpected moderate-or-higher advisory set");
  const advisory = actionable[0];
  assert.equal(advisory.github_advisory_id, advisoryId);
  assert.equal(advisory.module_name, "node-forge");
  assert.equal(advisory.severity, "high");
  assert.equal(advisory.url, "https://github.com/advisories/" + advisoryId);
  assert.equal(advisory.vulnerable_versions, "<=1.4.0");
  assert.equal(advisory.patched_versions, null);
  assert.ok(Array.isArray(advisory.findings) && advisory.findings.length > 0);
  for (const findingValue of advisory.findings) {
    const finding = record(findingValue);
    assert.equal(finding.version, "1.4.0");
    assert.equal(finding.dev, false);
    assert.equal(finding.bundled, false);
    assert.equal(typeof finding.optional, "boolean");
    assert.ok(Array.isArray(finding.paths) && finding.paths.length > 0);
    for (const trail of finding.paths) {
      assert.ok(
        typeof trail === "string" && trail.endsWith(">listhen>node-forge"),
        "new Forge consumer",
      );
    }
  }
  assert.equal(counts.high, 1);
  assert.equal(counts.moderate, 0);
  assert.equal(counts.critical, 0);
}

export function assertCleanReport(value: unknown): void {
  const { actionable, counts } = auditEntries(value);
  assert.equal(actionable.length, 0);
  assert.equal(counts.high, 0);
  assert.equal(counts.moderate, 0);
  assert.equal(counts.critical, 0);
}

export function packageDigest(packageRoot: string): string {
  const files: [string, string][] = [];
  function walk(directory: string): void {
    for (const name of fs.readdirSync(directory).sort()) {
      const file = path.join(directory, name);
      const stat = fs.lstatSync(file);
      if (stat.isDirectory()) walk(file);
      else {
        assert.ok(stat.isFile(), "unexpected package symlink");
        files.push([
          path.relative(packageRoot, file).split(path.sep).join("/"),
          createHash("sha256").update(fs.readFileSync(file)).digest("hex"),
        ]);
      }
    }
  }
  walk(packageRoot);
  files.sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0));
  return createHash("sha256").update(JSON.stringify(files)).digest("hex");
}

export function assertNoDirectForge(source: string): void {
  assertNoPackageConsumers(source);
}

export function verifyPatchFile(root: string): void {
  assert.equal(
    createHash("sha256")
      .update(fs.readFileSync(path.join(root, patchPath)))
      .digest("hex"),
    patchHash,
  );
}

export function assertForgePackage(packageRoot: string): void {
  packageRoot = fs.realpathSync(packageRoot);
  const require = createRequire(path.join(packageRoot, "package.json"));
  const manifest = record(
    JSON.parse(fs.readFileSync(path.join(packageRoot, "package.json"), "utf8")),
  );
  assert.equal(manifest.name, "node-forge");
  assert.equal(manifest.version, "1.4.0");
  assert.equal(manifest.main, "lib/index.js");
  assert.equal(packageDigest(packageRoot), forgeHash, "unpatched or modified Forge source");
  assert.equal(
    createHash("sha256")
      .update(fs.readFileSync(path.join(packageRoot, "lib/rsa.js")))
      .digest("hex"),
    rsaHash,
  );
  assert.equal(require.resolve(packageRoot), path.join(packageRoot, "lib/index.js"));
}

export function verifyInstalledForge(root: string): {
  instances: number;
  patch: string;
  rsa: string;
} {
  const workspaceDocument = YAML.parseDocument(
    fs.readFileSync(path.join(root, "pnpm-workspace.yaml"), "utf8"),
  );
  assert.equal(workspaceDocument.errors.length, 0);
  const documents = YAML.parseAllDocuments(
    fs.readFileSync(path.join(root, "pnpm-lock.yaml"), "utf8"),
  );
  assert.equal(documents.length, 2);
  assert.ok(documents.every((document) => document.errors.length === 0));
  assertForgeLock(workspaceDocument.toJS(), documents[1].toJS());
  verifyPatchFile(root);
  const tracked = spawnSync("git", ["ls-files", "-z"], {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 8 * 1024 * 1024,
  });
  assert.equal(tracked.status, 0, tracked.stderr);
  for (const relative of tracked.stdout
    .split("\0")
    .filter((file) => /\.(?:[cm]?js|[cm]?ts|jsx|tsx|vue|html)$/.test(file))) {
    assertNoDirectForge(fs.readFileSync(path.join(root, relative), "utf8"));
  }
  const store = path.join(root, "node_modules/.pnpm");
  const instances = new Set<string>();
  let listeners = 0;
  for (const directory of fs.readdirSync(store)) {
    if (!/^(?:node-forge@|listhen@)/.test(directory)) continue;
    const name = directory.startsWith("node-forge@") ? "node-forge" : "listhen";
    const packageRoot = fs.realpathSync(path.join(store, directory, "node_modules", name));
    const require = createRequire(path.join(packageRoot, "package.json"));
    const manifest = record(
      JSON.parse(fs.readFileSync(path.join(packageRoot, "package.json"), "utf8")),
    );
    assert.equal(manifest.name, name);
    assert.equal(manifest.version, name === "node-forge" ? "1.4.0" : "1.10.1");
    if (name === "listhen") {
      assert.equal(packageDigest(packageRoot), listhenHash, "unreviewed listhen source");
      const dependencyRoot = path.dirname(require.resolve("node-forge/package.json"));
      instances.add(fs.realpathSync(dependencyRoot));
      listeners++;
    } else instances.add(packageRoot);
  }
  assert.ok(listeners > 0 && instances.size > 0, "missing real listhen/Forge installation");
  for (const packageRoot of instances) {
    const require = createRequire(path.join(packageRoot, "package.json"));
    assertForgePackage(packageRoot);
    verifyForgeSecurity(require(packageRoot));
  }
  return { instances: instances.size, patch: patchHash, rsa: rsaHash };
}
