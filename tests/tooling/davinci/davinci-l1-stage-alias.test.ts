import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { parse as parseToml } from "@iarna/toml";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");

function readRepoFile(...parts: string[]): string {
  return fs.readFileSync(path.join(repoRoot, ...parts), "utf8");
}

test("Davinci L1 uses the physical crate package and directory", () => {
  const workspaceManifest = readRepoFile("Cargo.toml");
  assert.match(workspaceManifest, /^\s*"crates\/vize_l1",$/m);
  const manifest = parseToml(workspaceManifest) as {
    workspace?: { dependencies?: { vize_l1?: unknown }; package?: { version?: unknown } };
  };
  const workspaceVersion = manifest.workspace?.package?.version;
  if (typeof workspaceVersion !== "string") {
    assert.fail("workspace package version must be declared");
  }
  assert.deepEqual(manifest.workspace?.dependencies?.vize_l1, {
    path: "crates/vize_l1",
    version: `=${workspaceVersion}`,
  });
  assert.doesNotMatch(workspaceManifest, /crates\/vize_sinopia/u);
  assert.doesNotMatch(workspaceManifest, /^vize_sinopia = /m);

  const surfaceManifest = readRepoFile("crates", "vize_l1", "Cargo.toml");
  assert.match(surfaceManifest, /^name = "vize_l1"$/m);

  const lockfile = readRepoFile("Cargo.lock");
  assert.match(lockfile, /^name = "vize_l1"$/m);
});

test("current lossless L1 recipes use the physical package and preserve historical proof", () => {
  const suites = readRepoFile("docs/davinci", "plan", "test-suites.md");
  assert.match(suites, /cargo test -p vize_l1 --test pug_fidelity/u);
  const task = readRepoFile("docs/davinci", "plan", "phase-2-tasks.md");
  assert.match(task, /## P2-7 — L1 Vue surface tree/u);
  const record = readRepoFile("docs/davinci", "plan", "phase-2-records", "p2-7.md");
  assert.match(record, /cargo test -p vize_s1 --features davinci-differential/u);
  assert.match(record, /cargo tree -i vize_s1\s+--workspace/u);
  assert.doesNotMatch(record, /cargo (?:test -p|tree -i) vize_sinopia/u);
  assert.doesNotMatch(record, /crates\/vize_sinopia/u);
});
