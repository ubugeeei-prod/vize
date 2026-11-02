import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { existsSync, lstatSync, readFileSync, realpathSync } from "node:fs";
import { dirname, join } from "node:path";
import { asRecord } from "../../../support/json-value.ts";
import { fileImage, inside } from "../../../support/release/computed_event_custody.ts";
import type { PackageBase } from "../../../support/release/human_reports_11252_11253_public/model.ts";

const programs: Record<string, string> = {
  "package.json": "29955061fde038b8154a844f4d99837343143d057ef8feb2a588bbf5c7d2cbd1",
  "bin/tsc": "2219f428a7e55aaf1f7ad85b9b0f0cf5078aeb76ccc9a7c6036c92d48f492ffd",
  "lib/tsc.js": "cbdfbf11c26ed00dfc073155e316a42d6d6d8a387be61006b82fa9aa93ac572e",
  "lib/getExePath.js": "5d0997c7466b0fe3b4e7c5a8f378bfc625e0dc39de49dee8fee6bb6234b294d7",
};
const parentIntegrity =
  "sha512-cjWjZAJDsmZzsNC5GwHNlW0YPM65jZa5oR0KyQcYX3KSVIfTvtBq//vewISyatKk4OLi5grZjHhIVE9fdrw1UQ==";
const targetIntegrity =
  "sha512-kcD51ihf+pnI5KT4uQQYG04OpwznaZYBlIQhDhl8fflFd27NktxXsUeguxZyvUMyWfum7IF2bBS6Jv7ox0AgFQ==";
const present = (path: string) => {
  try {
    lstatSync(path);
    return true;
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === "ENOENT") return false;
    throw error;
  }
};
/** Only non-executing module lookup is used; the caller first authenticates all
 * shipped manifests/programs against the canonical package SRI and source pins. */
export function assertTscModuleResolution(root: string, parent: string, target: string) {
  const manifest = asRecord(JSON.parse(readFileSync(join(parent, "package.json"), "utf8")));
  assert.equal(manifest.name, "typescript");
  assert.equal(manifest.version, "7.1.0-beta");
  assert.equal(manifest.type, "module");
  assert.deepEqual(manifest.bin, { tsc: "./bin/tsc" });
  assert.equal(asRecord(manifest.imports)["#getExePath"], "./lib/getExePath.js");
  assert.equal(
    asRecord(manifest.optionalDependencies)["@typescript/typescript-linux-x64"],
    "7.1.0-beta",
  );
  for (const scope of ["bin/package.json", "lib/package.json"])
    assert(!present(join(parent, scope)), "unexpected nested module scope refuses");
  const lib = join(parent, "lib").replaceAll("\\", "/");
  assert(
    !lib.endsWith("/packages/typescript/lib") && !lib.endsWith("/built/npm/typescript/lib"),
    "only the published installed resolver branch is declared",
  );
  const targetManifestPath = join(target, "package.json"),
    targetManifest = asRecord(JSON.parse(readFileSync(targetManifestPath, "utf8")));
  assert.equal(targetManifest.name, "@typescript/typescript-linux-x64");
  assert.equal(targetManifest.version, "7.1.0-beta");
  assert.deepEqual(targetManifest.exports, { "./package.json": "./package.json" });
  assert(!Object.hasOwn(targetManifest, "imports"));
  assert.deepEqual(targetManifest.os, ["linux"]);
  assert.deepEqual(targetManifest.cpu, ["x64"]);
  let directory = join(parent, "lib");
  for (;;) {
    const candidate = join(directory, "node_modules", "@typescript/typescript-linux-x64");
    if (candidate === target) break;
    assert(!present(candidate), "prioritized compiler package shadow refuses");
    assert(inside(root, directory), "expected compiler package resolution base absent");
    directory = dirname(directory);
  }
  const resolver = createRequire(join(parent, "lib/getExePath.js"));
  assert.equal(
    resolver.resolve("@typescript/typescript-linux-x64/package.json"),
    targetManifestPath,
  );
}
export function resolvedTscPackage<T extends { name: string; directory: string }>(
  parent: string,
  packages: T[],
) {
  const resolved = createRequire(join(parent, "lib/getExePath.js")).resolve(
    "@typescript/typescript-linux-x64/package.json",
  );
  const matches = packages.filter(
    (pkg) =>
      pkg.name === "@typescript/typescript-linux-x64" &&
      join(pkg.directory, "package.json") === resolved,
  );
  assert.equal(
    matches.length,
    1,
    "resolved compiler target must have authenticated package custody",
  );
  return matches[0]!;
}
export function publishedTscRoute(root: string, packages: PackageBase[]) {
  assert.equal(process.platform, "linux");
  assert.equal(process.arch, "x64");
  const one = (name: string) => {
    const rows = packages.filter((pkg) => pkg.name === name);
    assert.equal(rows.length, 1);
    return rows[0]!;
  };
  const parent = one("typescript");
  assert.equal(parent.version, "7.1.0-beta");
  assert.equal(parent.integrity, parentIntegrity);
  assert.equal(
    parent.archiveSha512,
    Buffer.from(parentIntegrity.slice(7), "base64").toString("hex"),
  );
  for (const [path, digest] of Object.entries(programs)) {
    const absolute = join(parent.directory, path),
      stat = lstatSync(absolute);
    assert(stat.isFile() && !stat.isSymbolicLink() && realpathSync(absolute) === absolute);
    const approved = parent.files.find((file) => file.path === absolute);
    assert(approved);
    assert.equal(approved.sha256, digest);
    assert.deepEqual(fileImage(absolute), approved);
  }
  const target = resolvedTscPackage(parent.directory, packages);
  assert.equal(target.version, parent.version);
  assert.equal(target.integrity, targetIntegrity);
  assert.equal(
    target.archiveSha512,
    Buffer.from(targetIntegrity.slice(7), "base64").toString("hex"),
  );
  const targetManifestPath = join(target.directory, "package.json");
  assert.equal(
    fileImage(targetManifestPath).sha256,
    "96557a8b3bdb4bd67f2781b3b217d935c063bb43b4791e3c0e0f1c04c6b1c0f9",
  );
  assertTscModuleResolution(root, parent.directory, target.directory);
  const executable = join(target.directory, "lib/tsc"),
    stat = lstatSync(executable);
  assert(
    stat.isFile() &&
      !stat.isSymbolicLink() &&
      realpathSync(executable) === executable &&
      (stat.mode & 0o111) !== 0,
  );
  const approved = target.files.find((file) => file.path === executable);
  assert(approved);
  assert.deepEqual(fileImage(executable), approved);
  assert(!present(join(root, "node_modules/.bin/node")), "local Node shadow refuses");
  const nodes = (process.env.PATH ?? "")
    .split(":")
    .map((path) => join(path, "node"))
    .filter(existsSync);
  assert(nodes.length > 0);
  assert.equal(realpathSync(nodes[0]!), realpathSync(process.execPath));
  return {
    frontend: "typescript@7.1.0-beta tsc",
    parent: parent.directory,
    target: target.directory,
    executable: approved,
    node: fileImage(realpathSync(process.execPath)),
    sourceBranch:
      "installed package; unconditional package.json export closes import/require resolution",
    launcherExitLaw:
      "execve or execFileSync forwards literal native status; no numeric diagnostic status claimed",
    actualCompilerPid: null,
    actualExecveOrSpawn: "UNKNOWN",
    runtimeImageObserved: false,
  };
}
