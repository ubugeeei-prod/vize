import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

import {
  addMissingVariables,
  collectStdout,
  compareShortlex,
  createDirectory,
  isDirectory,
  isRegularFile,
  joinPath,
  readDirectoryNames,
  readTextFile,
  runCommand,
  trimWhitespace,
  writeFileDurably,
} from "../../tools/support/command-runtime.ts";
import { repoRoot } from "./_helpers/tool-commands.ts";

void test("text helpers keep the previous trim, ordering and join semantics", () => {
  assert.equal(trimWhitespace(" \t\r\n value \n"), "value");
  assert.equal(trimWhitespace("\v value \f"), "\v value \f");
  assert.deepEqual(["b.node", "aa.node", "a.node", "zz", "z"].sort(compareShortlex), [
    "z",
    "zz",
    "a.node",
    "b.node",
    "aa.node",
  ]);
  assert.equal(joinPath("/usr/local", "/usr"), path.join("/usr/local", "usr"));
  assert.equal(joinPath("npm/native", "../../target"), "target");
  assert.equal(joinPath("", ""), ".");
});

void test("inherited variables keep precedence over added variables", () => {
  assert.deepEqual(addMissingVariables({ A: "added", B: "added" }, { A: "inherited", C: "" }), {
    A: "inherited",
    B: "added",
    C: "",
  });
});

void test("filesystem probes treat IO failures as absent", (t) => {
  const root = fs.mkdtempSync(path.join(tmpdir(), "command-runtime-"));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  fs.writeFileSync(path.join(root, "file"), "text");
  assert.equal(isDirectory(root), true);
  assert.equal(isDirectory(path.join(root, "file")), false);
  assert.equal(isRegularFile(path.join(root, "file")), true);
  assert.equal(isRegularFile(path.join(root, "missing")), false);
  assert.deepEqual(readDirectoryNames(path.join(root, "missing")), []);
  assert.equal(readTextFile(path.join(root, "file")), "text");
  assert.equal(readTextFile(path.join(root, "missing")), undefined);
});

void test("directory creation adds missing parents but rejects an existing target", (t) => {
  const root = fs.mkdtempSync(path.join(tmpdir(), "command-runtime-mkdir-"));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  createDirectory(path.join(root, "a/b/c"), 0o755);
  assert.equal(isDirectory(path.join(root, "a/b/c")), true);
  // Like the MoonBit helper, a trailing separator re-creates the parent it just made.
  assert.throws(() => createDirectory(`${path.join(root, "x/y")}/`, 0o755), { code: "EEXIST" });
  assert.equal(isDirectory(path.join(root, "x/y")), true);
  assert.throws(() => createDirectory(path.join(root, "a/b"), 0o755), { code: "EEXIST" });
  fs.writeFileSync(path.join(root, "file"), "");
  assert.throws(() => createDirectory(path.join(root, "file/child"), 0o755), { code: "ENOTDIR" });
  assert.throws(() => createDirectory("", 0o755), { code: "ENOENT" });
});

void test("durable writes create with a mode and truncate existing files in place", (t) => {
  const root = fs.mkdtempSync(path.join(tmpdir(), "command-runtime-write-"));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const target = path.join(root, "out.node");
  writeFileDurably(target, "first and longer", 0o600);
  writeFileDurably(target, Uint8Array.from([0x6f, 0x6b]), 0o755);
  assert.equal(fs.readFileSync(target, "utf8"), "ok");
  if (process.platform !== "win32") assert.equal(fs.statSync(target).mode & 0o777, 0o600);
});

void test("child processes report exit codes, signals and spawn failures", () => {
  // A Node child must not inherit this runner's private test-child marker.
  const { NODE_TEST_CONTEXT: _context, ...env } = process.env;
  const node = (source: string) => [process.execPath, ["-e", source], { env }] as const;
  assert.equal(runCommand(...node("process.exit(7)")), 7);
  const output = collectStdout(...node("process.stdout.write('日本語'); process.exit(3)"));
  assert.deepEqual(output, { exitCode: 3, stdout: "日本語" });
  if (process.platform !== "win32") {
    assert.equal(runCommand(...node("process.kill(process.pid, 'SIGTERM')")), 15);
  }
  assert.throws(() => runCommand(path.join(repoRoot, "missing-command"), []), { code: "ENOENT" });
  assert.throws(() => collectStdout("vize-missing-command", []), { code: "ENOENT" });
});

void test("an unexpected command error exits with status 1 and its message on stderr", () => {
  const runtime = pathToFileURL(path.join(repoRoot, "tools/support/command-runtime.ts")).href;
  const result = spawnSync(
    process.execPath,
    [
      "--input-type=module",
      "-e",
      `import { runEntrypoint } from ${JSON.stringify(runtime)}; runEntrypoint(() => { throw new Error("boom"); });`,
    ],
    { encoding: "utf8" },
  );
  assert.deepEqual([result.status, result.stdout, result.stderr], [1, "", "boom\n"]);
});
