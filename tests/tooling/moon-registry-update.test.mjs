import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { registryUpdateEnvironment } from "../../tools/support/release/moon-registry-update.mjs";
import {
  git,
  identity,
  installedMoon,
  refuseSymbolsProxy,
  registryFixture,
  runOwned,
} from "./support/moon-registry-fixture.mjs";

const repoRoot = fileURLToPath(new URL("../../", import.meta.url));
const wrapper = path.join(repoRoot, "tools/support/release/moon-registry-update.mjs");

test("registry refresh retains inherited Git settings and bounds only merge output", () => {
  const entries = {
    ...process.env,
    GIT_CONFIG_COUNT: "2",
    GIT_CONFIG_KEY_0: "vize.fixture-value",
    GIT_CONFIG_VALUE_0: "spaces, 'quotes', and 日本語",
    GIT_CONFIG_KEY_1: "merge.stat",
    GIT_CONFIG_VALUE_1: "true",
  };
  delete entries.GIT_CONFIG_PARAMETERS;
  for (const parameters of [undefined, "'merge.stat=true'"]) {
    const inherited = { ...entries, ...(parameters ? { GIT_CONFIG_PARAMETERS: parameters } : {}) };
    const before = { ...inherited };
    const updated = registryUpdateEnvironment(inherited);
    assert.deepEqual(inherited, before, "the caller environment must remain unchanged");
    for (const [key, value] of Object.entries(before)) {
      if (key !== "GIT_CONFIG_COUNT" && key !== "GIT_CONFIG_PARAMETERS")
        assert.equal(updated[key], value, key);
    }
    if (parameters) assert(updated.GIT_CONFIG_PARAMETERS.startsWith(parameters));
    for (const [operation, key, expected] of [
      ["--get-all", "vize.fixture-value", "spaces, 'quotes', and 日本語\n"],
      ["--get-all", "merge.stat", parameters ? "true\nfalse\ntrue\nfalse\n" : "true\nfalse\n"],
      ["--get", "merge.stat", "false\n"],
    ]) {
      const result = spawnSync(process.execPath, [wrapper, "git", "config", operation, key], {
        cwd: os.tmpdir(),
        env: inherited,
        encoding: "utf8",
      });
      assert.equal(result.error, undefined, result.stderr);
      assert.equal(result.status, 0, result.stderr);
      assert.equal(result.stdout, expected);
    }
  }
});

test("registry refresh accepts Git's count syntax and rejects unsafe additions without mutation", () => {
  for (const count of ["1", "01", "+1", " 1", "\t1", "-0", ""]) {
    const inherited = {
      ...process.env,
      GIT_CONFIG_COUNT: count,
      GIT_CONFIG_KEY_0: "vize.count-fixture",
      GIT_CONFIG_VALUE_0: "retained",
    };
    delete inherited.GIT_CONFIG_PARAMETERS;
    const before = { ...inherited };
    const original = spawnSync("git", ["config", "--get", "vize.count-fixture"], {
      cwd: os.tmpdir(),
      env: inherited,
      encoding: "utf8",
    });
    const hasEntry = Number(count) === 1;
    assert.equal(original.status, hasEntry ? 0 : 1, original.stderr);
    if (hasEntry) assert.equal(original.stdout, "retained\n");
    const bounded = spawnSync(process.execPath, [wrapper, "git", "config", "--get", "merge.stat"], {
      cwd: os.tmpdir(),
      env: inherited,
      encoding: "utf8",
    });
    assert.equal(bounded.error, undefined, bounded.stderr);
    assert.equal(bounded.status, 0, bounded.stderr);
    assert.equal(bounded.stdout, "false\n");
    assert.deepEqual(inherited, before);
  }
  for (const count of ["-1", "1 ", "1.5", "invalid", "2147483647", "9007199254740992"]) {
    const inherited = {
      GIT_CONFIG_COUNT: count,
      GIT_CONFIG_KEY_0: "vize.count-fixture",
      GIT_CONFIG_VALUE_0: "retained",
    };
    const before = { ...inherited };
    assert.throws(() => registryUpdateEnvironment(inherited), /Git configuration|GIT_CONFIG_COUNT/);
    assert.deepEqual(inherited, before);
  }
});

test(
  "real Moon refreshes the identical large stale index without a Git pipe deadlock",
  {
    skip: process.platform === "win32",
  },
  async () => {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-moon-registry-update-"));
    const evidenceDir = path.join(
      repoRoot,
      "target/vize-tests/moon-registry-update",
      path.basename(root),
    );
    fs.mkdirSync(evidenceDir, { recursive: true });
    let proxy;
    try {
      const moon = installedMoon(repoRoot);
      const compiler = spawnSync(path.join(path.dirname(moon), "moonc"), ["-v"], {
        encoding: "utf8",
      });
      assert.equal(compiler.status, 0, compiler.stderr);
      assert.equal(
        compiler.stdout.trim().split(/\s+/)[0],
        `v${fs.readFileSync(path.join(repoRoot, ".moonbit-version"), "utf8").trim()}`,
      );
      const cli = spawnSync(moon, ["version"], { encoding: "utf8" });
      assert.equal(cli.status, 0, cli.stderr);
      fs.writeFileSync(path.join(evidenceDir, "toolchain.txt"), cli.stdout + compiler.stdout);
      const fixtureEnv = {
        ...process.env,
        LC_ALL: "C",
        GIT_AUTHOR_DATE: "2026-10-10T00:00:00Z",
        GIT_COMMITTER_DATE: "2026-10-10T00:00:00Z",
        GIT_CONFIG_COUNT: "2",
        GIT_CONFIG_KEY_0: "merge.stat",
        GIT_CONFIG_VALUE_0: "true",
        GIT_CONFIG_KEY_1: "core.hooksPath",
        GIT_CONFIG_VALUE_1: "/dev/null",
      };
      const fixture = registryFixture(root, fixtureEnv);
      const drained = git(fixture.index, ["pull", "origin", "main"], fixtureEnv);
      fs.writeFileSync(path.join(evidenceDir, "drained-git.stdout"), drained.stdout);
      fs.writeFileSync(path.join(evidenceDir, "drained-git.stderr"), drained.stderr);
      assert(
        Buffer.byteLength(drained.stdout) > 700_000,
        "the complete original workload must exceed the unread pipe capacity",
      );
      assert.deepEqual(identity(fixture.index, fixtureEnv), fixture.target);
      git(fixture.index, ["reset", "--hard", "--quiet", fixture.seed.head], fixtureEnv);
      const boundedGit = git(
        fixture.index,
        ["pull", "origin", "main"],
        registryUpdateEnvironment(fixtureEnv),
      );
      fs.writeFileSync(path.join(evidenceDir, "bounded-git.stdout"), boundedGit.stdout);
      fs.writeFileSync(path.join(evidenceDir, "bounded-git.stderr"), boundedGit.stderr);
      assert(Buffer.byteLength(boundedGit.stdout) < 128);
      assert(Buffer.byteLength(boundedGit.stderr) < 4096);
      assert.deepEqual(identity(fixture.index, fixtureEnv), fixture.target);
      proxy = await refuseSymbolsProxy();
      const env = { ...fixtureEnv, ...proxy.env, MOON_HOME: fixture.home, MOON_BIN: moon };
      delete env.NODE_TEST_CONTEXT;
      const reset = () => {
        git(fixture.index, ["reset", "--hard", "--quiet", fixture.seed.head], fixtureEnv);
        assert.deepEqual(identity(fixture.index, fixtureEnv), fixture.seed);
      };
      reset();
      const red = await runOwned(moon, ["update"], { cwd: root, env, deadlineMs: 4_000 });
      fs.writeFileSync(path.join(evidenceDir, "original-moon.json"), JSON.stringify(red, null, 2));
      fs.writeFileSync(path.join(evidenceDir, "original-moon.stdout"), red.stdout);
      fs.writeFileSync(path.join(evidenceDir, "original-moon.stderr"), red.stderr);
      assert.equal(
        red.expired,
        true,
        "the pinned producer must reproduce its unread Git pipe failure",
      );
      assert.equal(red.stdout, "");
      assert.equal(red.stderr, "");
      assert(
        red.snapshot.some((entry) => entry.binary === "git"),
        "retain the actual waiting Git process",
      );
      if (process.platform === "linux") {
        assert(
          red.snapshot.some((entry) => entry.binary === "git" && /pipe_write/.test(entry.wait)),
          "the real Linux producer must be blocked writing its undrained pipe",
        );
      }
      assert.deepEqual(
        identity(fixture.index, fixtureEnv),
        fixture.target,
        "Git already completed the same fast-forward before blocking on its output",
      );
      assert.equal(proxy.requests(), 0, "the red producer blocks before the symbols request");
      reset();
      const green = await runOwned(process.execPath, [wrapper, moon, "update"], {
        cwd: root,
        env,
        deadlineMs: 10_000,
      });
      fs.writeFileSync(path.join(evidenceDir, "bounded-moon.json"), JSON.stringify(green, null, 2));
      fs.writeFileSync(path.join(evidenceDir, "bounded-moon.stdout"), green.stdout);
      fs.writeFileSync(path.join(evidenceDir, "bounded-moon.stderr"), green.stderr);
      assert.equal(green.expired, false, green.stderr);
      assert.equal(green.status, 0, green.stderr);
      assert.match(green.stderr, /Registry index updated successfully/);
      assert.match(green.stderr, /failed to update symbols/);
      assert(
        proxy.requests() > 0,
        "the original real producer still reaches its independent best-effort symbols request",
      );
      assert.deepEqual(identity(fixture.index, fixtureEnv), fixture.target);
      assert.deepEqual(identity(fixture.origin, fixtureEnv), fixture.target);
      fs.writeFileSync(
        path.join(evidenceDir, "identities.json"),
        JSON.stringify(
          {
            seed: fixture.seed,
            target: fixture.target,
            actual: identity(fixture.index, fixtureEnv),
            originalStdoutBytes: Buffer.byteLength(drained.stdout),
            originalStderrBytes: Buffer.byteLength(drained.stderr),
            boundedStdoutBytes: Buffer.byteLength(boundedGit.stdout),
            boundedStderrBytes: Buffer.byteLength(boundedGit.stderr),
          },
          null,
          2,
        ),
      );
      const transportEnv = { ...process.env };
      delete transportEnv.NODE_TEST_CONTEXT;
      const transport = spawnSync(
        process.execPath,
        [
          "tools/support/dependencies/export-capture-evidence.ts",
          evidenceDir,
          `${evidenceDir}.json.gz`,
          "moon-registry-update",
        ],
        { cwd: repoRoot, env: transportEnv, encoding: "utf8", maxBuffer: 4 * 1024 * 1024 },
      );
      assert.equal(transport.error, undefined, transport.stderr);
      assert.equal(transport.status, 0, transport.stderr);
      process.stdout.write(transport.stdout);
    } finally {
      if (proxy) await proxy.close();
      fs.rmSync(root, { recursive: true, force: true });
    }
  },
);
