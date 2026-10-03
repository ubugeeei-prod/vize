import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { parse as parseToml } from "@iarna/toml";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
const guests = "davinci/vize_extension_host/tests/guests";
const read = (file: string) => fs.readFileSync(path.join(root, file), "utf8");
type Manifest = { dependencies?: { vize_guest?: { path?: string } } };
type Lock = { package: { name: string; version: string }[] };

const candidates = [
  ...fs.readdirSync(path.join(root, guests)).map((name) => `${guests}/${name}/Cargo.toml`),
  "examples/volt-target/Cargo.toml",
];
const sdkManifests = candidates.filter((file) => {
  const manifest = parseToml(read(file)) as Manifest;
  const dependency = manifest.dependencies?.vize_guest?.path;
  return (
    dependency !== undefined &&
    path.resolve(root, path.dirname(file), dependency) === path.join(root, "davinci/vize_guest")
  );
});

test("release refreshes every standalone workspace using the in-tree guest SDK", () => {
  const source = read("tools/moon/cmd/release/apply_versions.mbt");
  const refreshed = [...source.matchAll(/"([^"]+\/Cargo\.toml)"/g)]
    .map((match) => match[1])
    .filter((file) => sdkManifests.includes(file));
  assert.deepEqual(refreshed.toSorted(), sdkManifests.toSorted());
});

test("standalone SDK locks retain the actual workspace guest version", () => {
  const workspace = parseToml(read("Cargo.toml")) as {
    workspace: { package: { version: string } };
  };
  const failures = sdkManifests.flatMap((manifest) => {
    const lock = parseToml(read(manifest.replace(/Cargo\.toml$/, "Cargo.lock"))) as Lock;
    const versions = lock.package.filter((entry) => entry.name === "vize_guest");
    return versions.length === 1 && versions[0].version === workspace.workspace.package.version
      ? []
      : [{ manifest, versions, expected: workspace.workspace.package.version }];
  });
  assert.deepEqual(failures, []);
});
