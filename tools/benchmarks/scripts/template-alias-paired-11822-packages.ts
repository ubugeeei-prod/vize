import type { PackageManifest } from "./template-alias-paired-11822-types.ts";
import assert from "node:assert/strict";
import { cpSync, existsSync, mkdirSync, readFileSync, realpathSync, symlinkSync } from "node:fs";
import { createHash } from "node:crypto";
import { dirname, join, relative } from "node:path";
import { assertPublishFiles } from "./template-alias-paired-11822-declarations.ts";

export function copyPublishedPackage(source: string, destination: string): PackageManifest {
  const manifest = JSON.parse(
    readFileSync(join(source, "package.json"), "utf8"),
  ) as PackageManifest;
  assertPublishFiles(source, manifest);
  mkdirSync(destination, { recursive: true });
  for (const name of ["package.json", ...(manifest.files ?? [])]) {
    assert.ok(
      !/[*!?{}]/.test(name) && !name.includes(".."),
      `unsupported publish file pattern ${name}`,
    );
    cpSync(join(source, name), join(destination, name), { recursive: true, dereference: false });
  }
  return manifest;
}

export function dependencyDirectory(from: string, name: string): string | null {
  let current = from;
  for (;;) {
    const path = join(current, "node_modules", name);
    if (existsSync(join(path, "package.json"))) return realpathSync(path);
    const parent = dirname(current);
    if (parent === current) return null;
    current = parent;
  }
}

export function requiredDependencyDirectory(from: string, name: string): string {
  const path = dependencyDirectory(from, name);
  assert.ok(path, `missing required source build dependency ${name}`);
  return path;
}

/** Copy the actually installed runtime dependency graph, keeping every link inside the capsule. */
export function stageRuntimeDependencies(
  source: string,
  destination: string,
  capsule: string,
  overrides: Record<string, string>,
  seen = new Map<string, string>(),
) {
  const manifest = JSON.parse(
    readFileSync(join(source, "package.json"), "utf8"),
  ) as PackageManifest;
  for (const [name, optional] of [
    ...Object.keys(manifest.dependencies ?? {}).map((name) => [name, false] as const),
    ...Object.keys(manifest.optionalDependencies ?? {}).map((name) => [name, true] as const),
  ]) {
    let target: string | undefined = overrides[name];
    if (!target) {
      const original = dependencyDirectory(source, name);
      if (!original && optional) continue;
      assert.ok(original, `missing installed runtime dependency ${name}`);
      target = seen.get(original);
      if (!target) {
        const id = createHash("sha256").update(original).digest("hex").slice(0, 20);
        target = join(capsule, ".dependencies", id);
        seen.set(original, target);
        cpSync(original, target, {
          recursive: true,
          dereference: false,
          filter: (path) => !relative(original, path).split("/").includes("node_modules"),
        });
        stageRuntimeDependencies(original, target, capsule, overrides, seen);
      }
    }
    const link = join(destination, "node_modules", name);
    mkdirSync(dirname(link), { recursive: true });
    symlinkSync(target, link, "dir");
  }
}

export function manifestChanges(original: PackageManifest, derived: PackageManifest) {
  const changes = [];
  for (const key of new Set([...Object.keys(original), ...Object.keys(derived)])) {
    if (JSON.stringify(original[key]) !== JSON.stringify(derived[key]))
      changes.push({ key, original: original[key], derived: derived[key] });
  }
  const allowed = [
    "dependencies",
    "devDependencies",
    "optionalDependencies",
    "peerDependencies",
  ] as const;
  assert.ok(
    changes.every(({ key }) => allowed.some((allowedKey) => allowedKey === key)),
    "publish normalization changed non-dependency manifest fields",
  );
  for (const section of allowed)
    for (const value of Object.values(derived[section] ?? {})) {
      assert.ok(
        typeof value === "string" && !/^(workspace|catalog):/.test(value),
        "unresolved source dependency protocol",
      );
    }
  return changes;
}
