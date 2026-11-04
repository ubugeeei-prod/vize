import assert from "node:assert/strict";
import { cpSync, existsSync, mkdirSync, readFileSync, realpathSync, symlinkSync } from "node:fs";
import { createHash } from "node:crypto";
import { dirname, join, relative } from "node:path";

export function copyPublishedPackage(source, destination) {
  const manifest = JSON.parse(readFileSync(join(source, "package.json"), "utf8"));
  mkdirSync(destination, { recursive: true });
  for (const name of ["package.json", ...(manifest.files ?? [])]) {
    assert.ok(
      !/[*!?{}]/.test(name) && !name.includes(".."),
      `unsupported publish file pattern ${name}`,
    );
    if (existsSync(join(source, name)))
      cpSync(join(source, name), join(destination, name), { recursive: true, dereference: false });
  }
  return manifest;
}

export function dependencyDirectory(from, name) {
  let current = from;
  for (;;) {
    const path = join(current, "node_modules", name);
    if (existsSync(join(path, "package.json"))) return realpathSync(path);
    const parent = dirname(current);
    if (parent === current) return null;
    current = parent;
  }
}

/** Copy the actually installed runtime dependency graph, keeping every link inside the capsule. */
export function stageRuntimeDependencies(
  source,
  destination,
  capsule,
  overrides,
  seen = new Map(),
) {
  const manifest = JSON.parse(readFileSync(join(source, "package.json"), "utf8"));
  for (const [name, optional] of [
    ...Object.keys(manifest.dependencies ?? {}).map((name) => [name, false]),
    ...Object.keys(manifest.optionalDependencies ?? {}).map((name) => [name, true]),
  ]) {
    let target = overrides[name];
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

export function manifestChanges(original, derived) {
  const changes = [];
  for (const key of new Set([...Object.keys(original), ...Object.keys(derived)])) {
    if (JSON.stringify(original[key]) !== JSON.stringify(derived[key]))
      changes.push({ key, original: original[key], derived: derived[key] });
  }
  const allowed = ["dependencies", "devDependencies", "optionalDependencies", "peerDependencies"];
  assert.ok(
    changes.every(({ key }) => allowed.includes(key)),
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
