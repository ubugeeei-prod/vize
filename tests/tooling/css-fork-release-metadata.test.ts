import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { repoRoot } from "./_helpers/tool-commands.ts";

const fixture = path.join(
  repoRoot,
  "tests/_fixtures/differential/css-engine-vendor-exit/release-inheritance/metadata",
);
const root = fs.readFileSync(path.join(fixture, "Cargo.toml"), "utf8");
const lock = fs.readFileSync(path.join(fixture, "Cargo.lock"), "utf8");
const checksum = (text: string) => createHash("sha256").update(text).digest("hex");

// This independent expected projection names only actual official version fields.
// The production verifier uses the original Rust metadata::rewrite, never this test projection.
function expected(target: string) {
  return {
    root: root
      .replace('version = "0.443.0"', `version = "${target}"`)
      .replaceAll('version = "=0.443.0"', `version = "=${target}"`),
    lock: lock.replace(
      /(name = "(?:vize(?:_[^"]+)?|davinci_harness|davinci_test_support|vize-zed-extension)"\nversion = ")0\.443\.0"/g,
      `$1${target}"`,
    ),
  };
}

test("whole official producer metadata is reversible and refuses every nonversion or incoherent cut", () => {
  assert.equal(checksum(root), "38babfd635c5af9df1d401b0b59923940310a405868cbb0772d9bfefbd7e6e68");
  assert.equal(checksum(lock), "194c3098330cbeb91e441a2f035fe3cfb37650195c6ff04baf88f09a08378684");
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-css-official-metadata-"));
  try {
    const invoke = (cargo: string, cargoLock: string) => {
      fs.writeFileSync(path.join(directory, "Cargo.toml"), cargo);
      fs.writeFileSync(path.join(directory, "Cargo.lock"), cargoLock);
      const result = spawnSync(
        "rust-script",
        [path.join(repoRoot, "tools/commands/release/pr.rs"), "verify-producer-metadata", fixture],
        { cwd: directory, encoding: "utf8", timeout: 180_000 },
      );
      assert.equal(result.error, undefined);
      assert.equal(result.signal, null);
      // This CLI must not write either input, including on refusal.
      assert.equal(fs.readFileSync(path.join(directory, "Cargo.toml"), "utf8"), cargo);
      assert.equal(fs.readFileSync(path.join(directory, "Cargo.lock"), "utf8"), cargoLock);
      return result;
    };
    for (const target of ["0.443.0", "0.448.0", "0.65535.0"]) {
      const input = expected(target);
      const result = invoke(input.root, input.lock);
      assert.equal(result.status, 0, result.stderr);
      assert.deepEqual(JSON.parse(result.stdout), {
        schema: 1,
        baselineVersion: "0.443.0",
        currentVersion: target,
        wholeForwardAndInverse: true,
        readOnly: true,
        publicationQualification: false,
        files: ["Cargo.toml", "Cargo.lock"].map((file) => ({
          path: file,
          changed: target !== "0.443.0",
        })),
      });
    }
    const next = expected("0.448.0");
    const cases = [
      { name: "root-only", root: next.root, lock },
      { name: "lock-only", root, lock: next.lock },
      { name: "mixed-minor", root: next.root, lock: expected("0.449.0").lock },
      { name: "root-nonversion", root: next.root + "# changed\n", lock: next.lock },
      { name: "lock-nonversion", root: next.root, lock: next.lock + "# changed\n" },
      {
        name: "provider-path",
        root: next.root.replace('path = "vendor/parcel_sourcemap"', 'path = "vendor/other"'),
        lock: next.lock,
      },
      {
        name: "fork-stale",
        root: next.root.replace(
          'package = "vize_parcel_sourcemap", version = "=0.448.0"',
          'package = "vize_parcel_sourcemap", version = "=0.443.0"',
        ),
        lock: next.lock,
      },
      {
        name: "one-own-lock-stale",
        root: next.root,
        lock: next.lock.replace(
          'name = "vize_parcel_sourcemap"\nversion = "0.448.0"',
          'name = "vize_parcel_sourcemap"\nversion = "0.443.0"',
        ),
      },
      ...["0.442.0", "0.448.1", "1.448.0", "0.0448.0", "0.448.0-rc.1"].map((target) => ({
        name: target,
        ...expected(target),
      })),
    ];
    for (const row of cases) {
      assert.notEqual(row.root + row.lock, next.root + next.lock, row.name);
      const result = invoke(row.root, row.lock);
      assert.equal(result.status, 1, row.name);
      assert.equal(result.stdout, "", row.name);
      assert.ok(result.stderr.length > 0, row.name);
    }
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
