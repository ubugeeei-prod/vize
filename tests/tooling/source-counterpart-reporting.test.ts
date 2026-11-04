import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test } from "node:test";
import { ownedCounterpartDirectory } from "../../tools/support/compat/github/source-counterpart-snapshot.mjs";
import {
  nodeTestCases,
  ownedPath,
  type NodeTestEvent,
} from "../../tools/support/compat/github/test-cases.ts";

function owner(check: (root: string) => void, ignored = true): void {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(tmpdir(), "vize-counterpart-report-")));
  const env = { ...process.env };
  for (const key of Object.keys(env)) if (key.startsWith("GIT_")) delete env[key];
  try {
    execFileSync("git", ["init", "--quiet"], { cwd: root, env });
    if (ignored) fs.writeFileSync(path.join(root, ".gitignore"), "target/\n");
    check(root);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
}

void test("literal counterpart tests remain reportable inside their owner's ignored output", () => {
  owner((root) => {
    const directory = ownedCounterpartDirectory(root);
    const file = path.join(directory, "tests/tooling/original.test.mjs");
    fs.mkdirSync(path.dirname(file), { recursive: true });
    fs.writeFileSync(file, "// unchanged historical source-only input\n");
    const relative = ownedPath(file, root);
    assert.ok(relative.startsWith("target/tooling-source-counterparts/"));
    const events: NodeTestEvent[] = ["test:pass", "test:fail"].map((type) => ({
      type,
      data: { name: "whole historical source law", file, details: { type: "test" } },
    }));
    assert.deepEqual(nodeTestCases(events, root), [
      { path: relative, name: `${relative} › whole historical source law`, outcome: "passed" },
      { path: relative, name: `${relative} › whole historical source law`, outcome: "failed" },
    ]);
    fs.rmSync(directory, { recursive: true });
    assert.equal(ownedPath(file, root), relative, "cleanup cannot lose a completed case");
    assert.throws(() => ownedPath(path.join(tmpdir(), "foreign.test.mjs"), root), /outside/);
  });
});

void test("counterparts require an ignored output boundary", () => {
  owner((root) => {
    assert.throws(() => ownedCounterpartDirectory(root));
    assert.equal(fs.existsSync(path.join(root, "target")), false);
  }, false);
});

void test("counterparts cannot follow an output symlink outside their owner", () => {
  owner((root) => {
    const external = fs.mkdtempSync(path.join(tmpdir(), "vize-foreign-counterpart-"));
    try {
      fs.symlinkSync(
        external,
        path.join(root, "target"),
        process.platform === "win32" ? "junction" : "dir",
      );
      assert.throws(() => ownedCounterpartDirectory(root));
      assert.deepEqual(fs.readdirSync(external), []);
    } finally {
      fs.rmSync(external, { recursive: true, force: true });
    }
  });
});
