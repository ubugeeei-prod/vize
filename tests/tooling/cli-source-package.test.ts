import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { cpSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { findNpmCli } from "../../tools/support/release/computed_event_names.ts";

const cli = fileURLToPath(new URL("../../npm/cli/", import.meta.url));
const privateShims = [
  "src/config/module-runtime.d.mts",
  "src/config/module-runtime.mjs",
  "src/config/vite-runtime.d.mts",
  "src/config/vite-runtime.mjs",
];
type PackedFile = { path: string; size: number; mode: number };

function packlist(directory: string): PackedFile[] {
  const args = [
    findNpmCli(),
    "pack",
    "--ignore-scripts",
    "--dry-run",
    "--json",
    "--offline",
    "--cache",
    path.join(directory, ".test-cache"),
    "--userconfig",
    path.join(directory, ".test-npmrc"),
  ];
  const result = spawnSync(process.execPath, args, {
    cwd: directory,
    encoding: "utf8",
    timeout: 30_000,
    maxBuffer: 4 * 1024 * 1024,
  });
  assert.equal(result.status, 0, result.stderr);
  const packets = JSON.parse(result.stdout) as { name: string; files: PackedFile[] }[];
  assert.equal(packets.length, 1);
  assert.equal(packets[0]!.name, "vize");
  return packets[0]!.files;
}

test("the packed CLI omits only the four repository-owned config loader shims", () => {
  const directory = mkdtempSync(path.join(tmpdir(), "vize-cli-source-package-"));
  try {
    cpSync(cli, directory, {
      recursive: true,
      filter: (source) => path.basename(source) !== "node_modules",
    });
    writeFileSync(path.join(directory, ".test-npmrc"), "");
    const actual = packlist(directory);
    rmSync(path.join(directory, "src/config/.npmignore"));
    const before = packlist(directory);
    assert.deepEqual(
      before.filter((file) => privateShims.includes(file.path)).map((file) => file.path),
      privateShims,
    );
    assert.deepEqual(
      actual,
      before.filter((file) => !privateShims.includes(file.path)),
    );
    assert.deepEqual(
      actual.filter((file) => file.path.startsWith("src/config/")).map((file) => file.path),
      ["src/config/pkl.d.ts", "src/config/pkl.js"],
    );
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});
