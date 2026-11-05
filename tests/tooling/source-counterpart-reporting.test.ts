import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import fs from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test } from "node:test";
import {
  ownedCounterpartDirectory,
  registerWithOwnedTemporaryDirectory,
  ownedCounterpartTemporaryRoot,
} from "../../tools/support/compat/github/source-counterpart-snapshot.mjs";
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

void test("nested literal temporary files remain owned without changing the reporting environment", () => {
  owner((root) => {
    const initial = tmpdir();
    const key = process.platform === "win32" ? "TEMP" : "TMPDIR";
    const preserved = Object.entries(process.env).filter(([name]) => name !== key);
    const temporary = ownedCounterpartTemporaryRoot(root);
    try {
      assert.equal(tmpdir(), temporary.directory);
      const child = fs.mkdtempSync(path.join(tmpdir(), "literal-original-"));
      const file = path.join(child, "source.test.mjs");
      fs.writeFileSync(file, "// unchanged literal source body\n");
      assert.ok(ownedPath(file, root).startsWith("target/tooling-source-counterparts/"));
      assert.ok(
        preserved.every(([name, value]) => process.env[name] === value),
        "reporting environment preserved",
      );
      assert.throws(() => ownedPath(path.join(initial, "foreign.test.mjs"), root), /outside/);
    } finally {
      temporary.release();
    }
    assert.equal(tmpdir(), initial);
    assert.equal(fs.existsSync(temporary.directory), false);
    assert.ok(preserved.every(([name, value]) => process.env[name] === value));
  });
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

void test("historical imports use owned temporary output and restore every environment value", async () => {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(tmpdir(), "vize-counterpart-register-")));
  const original = Object.fromEntries(
    ["TMPDIR", "TMP", "TEMP"].map((key) => [key, process.env[key]]),
  );
  try {
    execFileSync("git", ["init", "--quiet"], { cwd: root });
    fs.writeFileSync(path.join(root, ".gitignore"), "target/\n");
    const retained = await registerWithOwnedTemporaryDirectory(root, async () => {
      const nested = fs.mkdtempSync(path.join(tmpdir(), "vize-star-original-custody-"));
      const file = path.join(nested, "original.test.mjs");
      fs.writeFileSync(file, "// original literal import\n");
      return { nested, path: ownedPath(file, root) };
    });
    assert.ok(retained.value.path.startsWith("target/tooling-source-counterparts/"));
    assert.equal(fs.existsSync(retained.value.nested), true);
    assert.deepEqual(
      Object.fromEntries(Object.keys(original).map((key) => [key, process.env[key]])),
      original,
    );
    retained.cleanup();
    assert.equal(fs.existsSync(retained.value.nested), false);
    await assert.rejects(
      registerWithOwnedTemporaryDirectory(root, async () => {
        throw new Error("failed historical import");
      }),
      /failed historical import/,
    );
    assert.deepEqual(
      Object.fromEntries(Object.keys(original).map((key) => [key, process.env[key]])),
      original,
    );
    assert.deepEqual(fs.readdirSync(path.join(root, "target/tooling-source-counterparts")), []);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

void test("nested literal registrations retain report ownership without changing the reporter", async () => {
  const { provisionStarSourceCounterpart } =
    await import("../../tools/support/compat/github/source-counterpart-snapshot.mjs");
  const repoRoot = fs.realpathSync(new URL("../../", import.meta.url));
  const fixture = provisionStarSourceCounterpart(
    repoRoot,
    "bd5702c0a4d5fb3bb64487794fc45f6f876e6e38",
    ["package.json"],
  );
  const reporter = new URL(
    "../../tools/support/compat/github/test-case-reporter.ts",
    import.meta.url,
  ).pathname;
  const directories: string[] = [];
  const register = async () => {
    const directory = fs.mkdtempSync(path.join(tmpdir(), "vize-star-counterpart-nested-"));
    directories.push(directory);
    const file = path.join(directory, "literal.test.mjs");
    fs.writeFileSync(
      file,
      'import { test } from "node:test"; test("nested original case", () => {});\n',
    );
    const env: NodeJS.ProcessEnv = {
      ...process.env,
      VIZE_TEST_CASES_ROOT: repoRoot,
      TEST_CASES_JOB: "nested ownership control",
      GITHUB_RUN_ATTEMPT: "1",
    };
    delete env.NODE_TEST_CONTEXT;
    const observed = spawnSync(process.execPath, ["--test", "--test-reporter", reporter, file], {
      cwd: fixture.root,
      encoding: "utf8",
      env,
    });
    return { file, observed };
  };
  try {
    const before = await register();
    assert.equal(before.observed.status, 1);
    assert.match(before.observed.stderr, /is outside/);
    const after = await fixture.register(register);
    assert.equal(after.observed.status, 0, after.observed.stderr);
    assert.equal(after.observed.stderr, "");
    const relative = ownedPath(after.file, repoRoot);
    assert.deepEqual(JSON.parse(after.observed.stdout), {
      version: 1,
      job: "nested ownership control",
      attempt: 1,
      cases: [{ path: relative, name: `${relative} › nested original case`, outcome: "passed" }],
    });
    assert.ok(fs.existsSync(after.file), "registered source remains available until cleanup");
    fixture.cleanup();
    assert.equal(fs.existsSync(after.file), false);
  } finally {
    fixture.cleanup();
    for (const directory of directories) fs.rmSync(directory, { recursive: true, force: true });
  }
});
