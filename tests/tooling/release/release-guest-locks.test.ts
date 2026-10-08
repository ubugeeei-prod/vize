import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { parse as parseToml } from "@iarna/toml";
import { runRepositoryGuardFixture } from "../support/release-guard-fixture.ts";

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

test("real release preparation preserves every external byte in all four SDK locks", () => {
  assert.equal(sdkManifests.length, 4);
  const guestLocks = Object.fromEntries(
    sdkManifests.map((manifest) => {
      const lockPath = manifest.replace(/Cargo\.toml$/, "Cargo.lock");
      const original = read(lockPath);
      const lock = parseToml(original) as Lock;
      const guest = lock.package.filter((entry) => entry.name === "vize_guest");
      assert.equal(guest.length, 1);
      const marker = `name = "vize_guest"\nversion = "${guest[0].version}"`;
      assert.equal(original.split(marker).length, 2);
      return [lockPath, original.replace(marker, 'name = "vize_guest"\nversion = "0.290.0"')];
    }),
  );
  const fixture = runRepositoryGuardFixture({ branch: "main", guestLocks });
  try {
    assert.equal(
      fixture.result.status,
      0,
      `${fixture.result.error ?? ""}\n${fixture.result.stdout}\n${fixture.result.stderr}`,
    );
    for (const [lockPath, before] of Object.entries(guestLocks)) {
      const expected = before.replace(
        'name = "vize_guest"\nversion = "0.290.0"',
        'name = "vize_guest"\nversion = "0.290.1"',
      );
      assert.equal(
        fs.readFileSync(path.join(fixture.tempDir, lockPath), "utf8"),
        expected,
        lockPath,
      );
      assert.ok(fixture.gitLog.includes(lockPath), `${lockPath} was staged by the real producer`);
    }
  } finally {
    fs.rmSync(fixture.tempDir, { recursive: true, force: true });
  }
});
