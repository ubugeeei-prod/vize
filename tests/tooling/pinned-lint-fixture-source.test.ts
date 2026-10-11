import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import {
  assertPinnedFixtureFiles,
  exportPinnedFixtureGitlinks,
  readPinnedFixtureSource,
} from "../_helpers/pinned-fixture-source.ts";

function git(cwd: string, args: string[]): string {
  return execFileSync("git", args, { cwd, encoding: "utf8" }).trim();
}

function withFixture(run: (root: string, pin: string) => void): void {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-pinned-lint-"));
  try {
    git(root, ["init", "-q"]);
    fs.writeFileSync(path.join(root, "Original.vue"), "<template><div /></template>\n");
    fs.writeFileSync(path.join(root, "vite.config.ts"), "export default {}\n");
    fs.writeFileSync(path.join(root, "package.json"), '{"packageManager":"pnpm@10.28.2"}\n');
    fs.writeFileSync(path.join(root, "pnpm-lock.yaml"), "lockfileVersion: '9.0'\n");
    git(root, ["add", "."]);
    git(root, [
      "-c",
      "user.name=Vize",
      "-c",
      "user.email=vize@example.com",
      "commit",
      "-qm",
      "pin",
    ]);
    run(root, git(root, ["rev-parse", "HEAD"]));
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
}

test("an unhydrated fixture cannot authenticate its parent's Git HEAD", () => {
  withFixture((root, pin) => {
    const empty = path.join(root, "empty-submodule");
    fs.mkdirSync(empty);
    assert.throws(() => readPinnedFixtureSource(empty, pin), /initialized repository/);
    assert.throws(() => readPinnedFixtureSource(root, "0".repeat(40)), /owning Gitlink/);
  });
});

test("dependency setup may add installed files while all original blobs stay exact", () => {
  withFixture((root, pin) => {
    const source = readPinnedFixtureSource(root, pin);
    fs.mkdirSync(path.join(root, "node_modules"));
    fs.writeFileSync(path.join(root, "node_modules", "installed.json"), '{"version":"1.0.0"}');
    assert.equal(source.blobs.length, 4);
    assertPinnedFixtureFiles(root, source);
  });
});

test("nested Gitlinks must be hydrated at their own pin and their complete blobs are exported", () => {
  withFixture((root) => {
    const nested = path.join(root, "assets");
    fs.mkdirSync(nested);
    git(nested, ["init", "-q"]);
    fs.writeFileSync(path.join(nested, "original.txt"), "nested original\n");
    git(nested, ["add", "."]);
    git(nested, [
      "-c",
      "user.name=Vize",
      "-c",
      "user.email=vize@example.com",
      "commit",
      "-qm",
      "nested",
    ]);
    const nestedPin = git(nested, ["rev-parse", "HEAD"]);
    git(root, ["update-index", "--add", "--cacheinfo", "160000", nestedPin, "assets"]);
    git(root, [
      "-c",
      "user.name=Vize",
      "-c",
      "user.email=vize@example.com",
      "commit",
      "-qm",
      "gitlink",
    ]);
    const pin = git(root, ["rev-parse", "HEAD"]);
    const source = readPinnedFixtureSource(root, pin);
    assert.deepEqual(source.gitlinks, [{ path: "assets", revision: nestedPin }]);
    assertPinnedFixtureFiles(root, source);
    const exported = path.join(root, "exported");
    fs.mkdirSync(exported);
    for (const file of ["Original.vue", "vite.config.ts", "package.json", "pnpm-lock.yaml"]) {
      fs.copyFileSync(path.join(root, file), path.join(exported, file));
    }
    exportPinnedFixtureGitlinks(exported, root, source);
    assertPinnedFixtureFiles(exported, source);
    fs.appendFileSync(path.join(exported, "assets", "original.txt"), "changed");
    assert.throws(
      () => assertPinnedFixtureFiles(exported, source),
      /original fixture bytes changed/,
    );
    fs.rmSync(nested, { recursive: true });
    fs.mkdirSync(nested);
    assert.throws(() => readPinnedFixtureSource(root, pin), /initialized repository/);
  });
});

for (const file of ["Original.vue", "vite.config.ts", "package.json", "pnpm-lock.yaml"]) {
  test(`fixture verification rejects rewritten ${file}`, () => {
    withFixture((root, pin) => {
      const source = readPinnedFixtureSource(root, pin);
      fs.appendFileSync(path.join(root, file), "changed\n");
      assert.throws(() => assertPinnedFixtureFiles(root, source), /original fixture bytes changed/);
    });
  });
}

test(
  "a regular source replaced with a symlink does not inherit the target's identity",
  {
    skip: process.platform === "win32",
  },
  () => {
    withFixture((root, pin) => {
      const source = readPinnedFixtureSource(root, pin);
      const original = path.join(root, "Original.vue");
      const replacement = path.join(root, "replacement.vue");
      fs.copyFileSync(original, replacement);
      fs.unlinkSync(original);
      fs.symlinkSync(replacement, original);
      assert.throws(() => assertPinnedFixtureFiles(root, source), /changed type/);
    });
  },
);

test(
  "executable permissions are part of the authenticated fixture on Unix",
  {
    skip: process.platform === "win32",
  },
  () => {
    withFixture((root, pin) => {
      const source = readPinnedFixtureSource(root, pin);
      fs.chmodSync(path.join(root, "vite.config.ts"), 0o755);
      assert.throws(() => assertPinnedFixtureFiles(root, source), /executable mode changed/);
    });
  },
);
