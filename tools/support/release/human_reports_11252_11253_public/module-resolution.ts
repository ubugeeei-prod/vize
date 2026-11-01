import assert from "node:assert/strict";
import { lstatSync, realpathSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join, resolve } from "node:path";
import { fileImage } from "../computed_event_custody.ts";
import type { Image, PackageBase } from "./model.ts";
import type { Sink } from "./evidence.ts";

/** Node resolution only: never require(), import(), dlopen(), or execute a
 * candidate. Inspect physical candidates independently of Node's path cache. */
function resolveModule(
  base: string,
  specifier: string,
  owners: readonly PackageBase[],
  allowed: readonly Image[],
  sink: Sink,
): Image | null {
  const loader = createRequire(base);
  const extensions = [
    ...new Set([".js", ".json", ".node", ".ts", ".cts", ".mts", ...Object.keys(loader.extensions)]),
  ];
  assert(extensions.every((e) => /^\.[a-z]+$/u.test(e)));
  const relative = specifier.startsWith("./") || specifier.startsWith("../");
  const parts = specifier.split("/");
  const packageName = relative ? null : parts.slice(0, specifier.startsWith("@") ? 2 : 1).join("/");
  if (packageName) assert.match(packageName, /^(?:@[a-z0-9._-]+\/)?[a-z0-9._-]+$/u);
  const suffix = relative ? "" : parts.slice(specifier.startsWith("@") ? 2 : 1).join("/");
  const lookups = relative ? [dirname(base)] : loader.resolve.paths(packageName!);
  assert(lookups);
  let found: Image | null = null;
  const inspectFile = (path: string): boolean => {
    const stat = lstatSync(path, { throwIfNoEntry: false });
    sink("native-module-resolution-candidate", {
      base,
      specifier,
      path,
      present: stat !== undefined,
      kind: !stat
        ? null
        : stat.isSymbolicLink()
          ? "symlink"
          : stat.isFile()
            ? "file"
            : stat.isDirectory()
              ? "directory"
              : "other",
    });
    if (!stat) return false;
    const canonical = realpathSync(path),
      actualStat = lstatSync(canonical);
    if (!actualStat.isFile()) return false;
    const expected = allowed.filter((image) => image.path === canonical);
    assert.equal(
      expected.length,
      1,
      "unknown module/native resolution candidate refuses execution",
    );
    const actual = fileImage(canonical);
    assert.deepEqual(actual, expected[0]);
    found = actual;
    return true;
  };
  for (const lookup of lookups) {
    const packagePath = relative ? resolve(dirname(base), specifier) : join(lookup, packageName!);
    let expectedDirectory = false;
    const rootStat = lstatSync(packagePath, { throwIfNoEntry: false });
    if (rootStat && lstatSync(realpathSync(packagePath)).isDirectory()) {
      const directory = realpathSync(packagePath),
        matches = owners.filter((owner) => owner.directory === directory);
      assert.equal(matches.length, 1, "unknown package/directory candidate refuses execution");
      const manifest = matches[0]!.files.filter(
        (file) => file.path === join(directory, "package.json"),
      );
      assert.equal(manifest.length, 1);
      assert.deepEqual(fileImage(join(directory, "package.json")), manifest[0]);
      expectedDirectory = true;
    }
    // Check all file alternatives even when .resolve() has a cached answer.
    // An unexpected directory fallback is refused above without loading it.
    const fileBase = suffix ? join(packagePath, suffix) : packagePath;
    if (!expectedDirectory || suffix) inspectFile(fileBase);
    for (const extension of extensions) inspectFile(fileBase + extension);
    if (expectedDirectory || found) break;
  }
  let actual: string | null = null;
  try {
    actual = loader.resolve(specifier);
  } catch (error) {
    const code = typeof error === "object" && error !== null && "code" in error ? error.code : null;
    sink("native-module-resolution-result", { base, specifier, resolved: null, code });
    assert.equal(
      code,
      "MODULE_NOT_FOUND",
      "unclassified module-resolution error refuses execution",
    );
  }
  if (actual === null) {
    assert.equal(found, null, "cached resolution contradicts a physical candidate");
    return null;
  }
  const canonical = realpathSync(actual),
    expected = allowed.filter((file) => file.path === canonical);
  assert.equal(
    expected.length,
    1,
    "resolved module is not the expected authenticated image/member",
  );
  const image = fileImage(canonical);
  assert.deepEqual(image, expected[0]);
  if (found) assert.deepEqual(image, found);
  sink("native-module-resolution-result", { base, specifier, resolved: image, code: null });
  return image;
}
export function resolveAuthenticatedModule(
  base: string,
  specifier: string,
  owners: readonly PackageBase[],
  allowed: readonly Image[],
  sink: Sink,
): Image | null {
  const observations: Array<{ kind: string; value: unknown }> = [];
  try {
    const result = resolveModule(base, specifier, owners, allowed, (kind, value) =>
      observations.push({ kind, value }),
    );
    sink("native-module-resolution-observations", {
      base,
      specifier,
      observations,
      complete: true,
      result,
    });
    return result;
  } catch (error) {
    sink("native-module-resolution-observations", {
      base,
      specifier,
      observations,
      complete: false,
      errorKind: error instanceof Error ? error.name : typeof error,
    });
    throw error;
  }
}
