import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

function readRepoFile(...parts: string[]): string {
  return fs.readFileSync(path.join(repoRoot, ...parts), "utf8");
}

test("Davinci S1 uses the physical crate package and directory", () => {
  const workspaceManifest = readRepoFile("Cargo.toml");
  assert.match(workspaceManifest, /^\s*"crates\/vize_s1",$/m);
  assert.match(
    workspaceManifest,
    /^vize_s1 = \{ path = "crates\/vize_s1", version = "=0\.390\.0" \}$/m,
  );
  assert.doesNotMatch(workspaceManifest, /crates\/vize_sinopia/u);
  assert.doesNotMatch(workspaceManifest, /^vize_sinopia = /m);

  const surfaceManifest = readRepoFile("crates", "vize_s1", "Cargo.toml");
  assert.match(surfaceManifest, /^name = "vize_s1"$/m);

  const lockfile = readRepoFile("Cargo.lock");
  assert.match(lockfile, /^name = "vize_s1"$/m);
});

test("P2-7 lossless S1 recipes use the physical crate package", () => {
  const record = readRepoFile("davinci-road", "plan", "phase-2-records", "p2-7.md");
  assert.match(record, /cargo test -p vize_s1 --features davinci-differential/u);
  assert.match(record, /cargo tree -i vize_s1\s+--workspace/u);
  assert.doesNotMatch(record, /cargo (?:test -p|tree -i) vize_sinopia/u);
  assert.doesNotMatch(record, /crates\/vize_sinopia/u);
});
