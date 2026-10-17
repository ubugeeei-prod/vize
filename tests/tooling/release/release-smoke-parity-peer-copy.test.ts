import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { COPY_TREE_SOURCE } from "./release-smoke-parity-peer.ts";

type Entry = { name: string; type: string; mode: number; bytes?: string; target?: string };

function tree(directory: string): Entry[] {
  if (!fs.existsSync(directory)) return [];
  const entries: Entry[] = [];
  const visit = (filename: string, name: string) => {
    const stat = fs.lstatSync(filename);
    const mode = stat.mode & 0o777;
    if (stat.isSymbolicLink()) {
      entries.push({ name, type: "symlink", mode, target: fs.readlinkSync(filename) });
    } else if (stat.isDirectory()) {
      entries.push({ name, type: "directory", mode });
      for (const child of fs.readdirSync(filename).sort()) {
        visit(path.join(filename, child), name ? `${name}/${child}` : child);
      }
    } else {
      assert.ok(stat.isFile(), filename);
      entries.push({
        name,
        type: "file",
        mode,
        bytes: fs.readFileSync(filename).toString("base64"),
      });
    }
  };
  visit(directory, "");
  return entries;
}

function contains(parent: string, child: string): boolean {
  const relative = path.relative(parent, child);
  return (
    relative === "" ||
    (!path.isAbsolute(relative) && relative !== ".." && !relative.startsWith(`..${path.sep}`))
  );
}

for (const rich of [false, true]) {
  test(`npm parity peer copy isolates recursive cp baseline and preserves complete tree (${rich ? "nested" : "original"})`, () => {
    const temporary = fs.realpathSync.native(
      fs.mkdtempSync(path.join(os.tmpdir(), "vize-smoke-copy-")),
    );
    try {
      const fixture = path.join(temporary, "rust fixture 新規");
      const source = path.join(fixture, "repo/npm/0");
      const install = path.join(fixture, "temp/vize-release-smoke-Control/install");
      const destination = path.join(install, "node_modules/@vizejs/parity");
      fs.mkdirSync(source, { recursive: true });
      fs.mkdirSync(install, { recursive: true });
      fs.writeFileSync(
        path.join(source, "package.json"),
        JSON.stringify(
          {
            name: "@vizejs/parity",
            version: "1.2.3",
            main: "./index.js",
            bin: { parity: "cli.js" },
            dependencies: { retained: "1.0.0" },
          },
          null,
          2,
        ) + "\n",
      );
      fs.writeFileSync(path.join(source, "index.js"), "module.exports = { ok: true };\n");
      fs.writeFileSync(
        path.join(source, "cli.js"),
        "#!/usr/bin/env node\nconsole.log('parity executable');\n",
        { mode: 0o755 },
      );
      if (rich) {
        fs.mkdirSync(path.join(source, "nested/empty"), { recursive: true });
        fs.writeFileSync(path.join(source, "nested/.binary"), Buffer.from([0, 1, 127, 128, 255]));
        fs.chmodSync(path.join(source, "nested/.binary"), 0o640);
      }
      const before = tree(source);
      assert.ok(before.length >= 4);
      assert.ok(before.every((entry) => entry.type !== "symlink"));
      assert.equal(contains(source, destination), false);
      assert.equal(contains(destination, source), false);
      const ancestry = {
        source: fs.realpathSync.native(source),
        destination,
        sourceToDestination: path.relative(source, destination),
        destinationToSource: path.relative(destination, source),
        sourceContainsDestination: contains(source, destination),
        destinationContainsSource: contains(destination, source),
      };
      const baseline = spawnSync(
        process.execPath,
        [
          "-e",
          'require("node:fs").cpSync(process.argv[1], process.argv[2], { recursive: true });',
          source,
          destination,
        ],
        { cwd: install, encoding: "utf8", timeout: 30_000 },
      );
      const baselineRecord = {
        kind: "npm-parity-peer-copy-baseline",
        node: process.version,
        platform: process.platform,
        ancestry,
        sourceTree: before,
        destinationTree: tree(destination),
        status: baseline.status,
        signal: baseline.signal,
        stdout: baseline.stdout,
        stderr: baseline.stderr,
        error: baseline.error?.message ?? null,
      };
      // The exact same path shape is executed without either release driver.
      // Only this explicit Windows native crash can explain a peer-only failure.
      console.log(JSON.stringify(baselineRecord));
      assert.ifError(baseline.error);
      assert.equal(baseline.signal, null);
      assert.equal(baseline.stdout, "");
      assert.equal(baseline.stderr, "");
      if (baseline.status === 0) {
        assert.deepEqual(tree(destination), before);
      } else {
        assert.equal(process.platform, "win32", JSON.stringify(baselineRecord));
        assert.ok(baseline.status !== null);
        assert.equal(baseline.status >>> 0, 0xc0000409, JSON.stringify(baselineRecord));
        console.log(
          JSON.stringify({ ...baselineRecord, kind: "npm-parity-peer-only-baseline-failure" }),
        );
      }
      fs.rmSync(destination, { recursive: true, force: true });
      const replacement = spawnSync(
        process.execPath,
        [
          "-e",
          `const fs = require("node:fs"); const path = require("node:path"); ${COPY_TREE_SOURCE}\ncopyTree(process.argv[1], process.argv[2]);`,
          source,
          destination,
        ],
        { cwd: install, encoding: "utf8", timeout: 30_000 },
      );
      const after = tree(destination);
      console.log(
        JSON.stringify({
          kind: "npm-parity-peer-copy-replacement",
          ancestry,
          status: replacement.status,
          signal: replacement.signal,
          stdout: replacement.stdout,
          stderr: replacement.stderr,
          destinationTree: after,
        }),
      );
      assert.ifError(replacement.error);
      assert.equal(replacement.status, 0);
      assert.equal(replacement.signal, null);
      assert.equal(replacement.stdout, "");
      assert.equal(replacement.stderr, "");
      assert.deepEqual(tree(source), before);
      assert.deepEqual(after, before);
    } finally {
      fs.rmSync(temporary, { recursive: true, force: true });
    }
  });
}

test("npm parity peer rejects recursive source/destination ancestry", () => {
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "vize-smoke-copy-ancestry-"));
  try {
    const destination = path.join(temporary, "nested");
    const result = spawnSync(
      process.execPath,
      [
        "-e",
        `const fs = require("node:fs"); const path = require("node:path"); ${COPY_TREE_SOURCE}\ntry { copyTree(process.argv[1], process.argv[2]); } catch (error) { process.stderr.write(error.message); process.exit(1); }`,
        temporary,
        destination,
      ],
      { encoding: "utf8" },
    );
    assert.ifError(result.error);
    assert.equal(result.status, 1);
    assert.equal(result.stdout, "");
    assert.equal(result.stderr, "fixture copy paths must be disjoint");
    assert.equal(fs.existsSync(destination), false);
  } finally {
    fs.rmSync(temporary, { recursive: true, force: true });
  }
});
