import assert from "node:assert/strict";
import { protectedBracesModule } from "./braces-consumers.ts";

export const bracesAdvisoryId = "GHSA-vfj7-8cjw-p6xm";
export const bracesPatchPath = "patches/braces@3.0.3.patch";
export const bracesPatchHash = "4164fc10a66e95450b372dda14add9e336b8daa6ef49d472caed0f6d9bc28d74";
const integrity =
  "sha512-yQbXgO/OSZVD2IsiLlro+7Hf6Q18EJrKSEsdoMzKePKXct3gvD8oLcOQdIzGupr5Fj+EDe8gO/lxc1BzfMpxvA==";
const patchedRef = "3.0.3(patch_hash=" + bracesPatchHash + ")";
const registryPackages = {
  "fill-range@7.1.1":
    "sha512-YsGpe3WHLK8ZYi4tWDg2Jy3ebRz2rXowDxnld4bkQB00cc/1Zw9AWnC0i9ztDJitivtQvaI9KaLyKrc+hBW0yg==",
  "to-regex-range@5.0.1":
    "sha512-65P7iz6X5yEr1cwcgvQxbbIw7Uk3gOy5dIdtZ4rDveLqhrdJP+Li/Hx6tyK0NEb+2GCyneCMJiGqrADCSNk8sQ==",
  "is-number@7.0.0":
    "sha512-41Cifkg6e8TylSpdtTpeLVMqvSBEVzTttHvERD741+pnZ8ANv0004MRL43QKPDlK9cGvNp6NZWZUBlbGXYxxng==",
  "chokidar@3.6.0":
    "sha512-7VT13fmjotKpGipCW9JEQAusEPE+Ei8nl6/g4FBAmIm0GOOLMua9NDDo/DWp0ZAxCr3cPq5ZpBqmPAQgDda2Pw==",
  "micromatch@4.0.8":
    "sha512-PXwfBhYu0hBCPw8Dn0E+WDYb7af3dSLVWKi3HGv84IdF4TyFoC0ysxFd0Goxw7nSv4T/PzEJQxsYsEiFCKo2BA==",
};

export function record(value: unknown): Record<string, unknown> {
  assert.ok(value && typeof value === "object" && !Array.isArray(value), "expected record");
  return value as Record<string, unknown>;
}

export function assertBracesLock(workspaceValue: unknown, lockValue: unknown): void {
  const workspace = record(workspaceValue),
    lock = record(lockValue);
  assert.ok(!workspace.audit, "audit suppression is outside this proof");
  assert.ok(!Object.keys(record(workspace.overrides ?? {})).some(protectedBracesModule));
  assert.ok(!Object.keys(record(lock.overrides ?? {})).some(protectedBracesModule));
  assert.equal(record(workspace.patchedDependencies)["braces@3.0.3"], bracesPatchPath);
  assert.deepEqual(
    Object.keys(record(workspace.patchedDependencies)).filter(protectedBracesModule),
    ["braces@3.0.3"],
  );
  assert.deepEqual(Object.keys(record(lock.patchedDependencies)).filter(protectedBracesModule), [
    "braces@3.0.3",
  ]);
  // The actual pnpm 12 producer records selector -> source hash. The separate
  // workspace declaration binds that hash to the reviewed patch path.
  assert.equal(record(lock.patchedDependencies)["braces@3.0.3"], bracesPatchHash);
  const packages = record(lock.packages),
    snapshots = record(lock.snapshots);
  assert.deepEqual(Object.keys(packages).filter(protectedBracesModule), ["braces@3.0.3"]);
  assert.deepEqual(record(packages["braces@3.0.3"]).resolution, { integrity });
  for (const [key, integrity] of Object.entries(registryPackages))
    assert.deepEqual(record(packages[key]).resolution, { integrity });
  assert.deepEqual(Object.keys(snapshots).filter(protectedBracesModule), ["braces@" + patchedRef]);
  assert.deepEqual(record(snapshots["braces@" + patchedRef]), {
    dependencies: { "fill-range": "7.1.1" },
  });
  assert.deepEqual(record(snapshots["fill-range@7.1.1"]), {
    dependencies: { "to-regex-range": "5.0.1" },
  });
  assert.deepEqual(record(snapshots["to-regex-range@5.0.1"]), {
    dependencies: { "is-number": "7.0.0" },
  });
  assert.deepEqual(record(snapshots["is-number@7.0.0"]), {});
  const parents: string[] = [];
  for (const [owner, snapshotValue] of Object.entries(snapshots)) {
    for (const field of ["dependencies", "optionalDependencies"]) {
      for (const [name, ref] of Object.entries(record(record(snapshotValue)[field] ?? {}))) {
        assert.equal(typeof ref, "string");
        if (!protectedBracesModule(name) && !protectedBracesModule(String(ref))) continue;
        assert.equal(name, "braces", "new aliased Braces consumer");
        assert.equal(ref, patchedRef, "unpatched Braces edge");
        parents.push(owner);
      }
    }
  }
  assert.deepEqual(parents.sort(), ["chokidar@3.6.0", "micromatch@4.0.8"]);
  for (const importerValue of Object.values(record(lock.importers))) {
    for (const field of ["dependencies", "devDependencies", "optionalDependencies"]) {
      for (const [name, value] of Object.entries(record(record(importerValue)[field] ?? {}))) {
        const dependency = record(value);
        assert.ok(
          ![name, dependency.specifier, dependency.version].some((value) =>
            protectedBracesModule(String(value)),
          ),
          "direct or aliased Braces importer",
        );
      }
    }
  }
}
