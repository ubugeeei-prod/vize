import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { commandExists } from "../../tools/vite-plus/root-build-task-plugin.ts";
import {
  getTaskShellLocaleAssignments,
  normalizeTaskShellLocale,
  shellCommand,
  shellCommandForwardingArguments,
  withRustTaskEnvironment,
} from "../../tools/vite-plus/task-shell.ts";
import {
  moonCommandForEnvironment,
  moonRegistryUpdateGuardForEnvironment,
} from "../../tools/vite-plus/task-commands.ts";
import { checkTasks } from "../../tools/vite-plus/tasks/check.ts";
import { releaseTasks } from "../../tools/vite-plus/tasks/release.ts";
import { writeFakeCommand } from "./support/fake-command.ts";

test("macOS task shells fall back from C.UTF-8 to an installed UTF-8 locale", () => {
  assert.deepEqual(
    getTaskShellLocaleAssignments("darwin", {
      LC_ALL: "C.UTF-8",
      LC_CTYPE: "C.UTF-8",
      LANG: "C.UTF-8",
    }),
    ["LC_ALL='en_US.UTF-8'", "LC_CTYPE='en_US.UTF-8'", "LANG='en_US.UTF-8'"],
  );
});

test("non-macOS task shells do not rewrite C.UTF-8", () => {
  assert.deepEqual(
    getTaskShellLocaleAssignments("linux", {
      LC_ALL: "C.UTF-8",
      LC_CTYPE: "C.UTF-8",
      LANG: "C.UTF-8",
    }),
    [],
  );
});

test("task shell commands apply locale before sh starts", () => {
  assert.equal(
    shellCommand("cd examples/vite-musea && pnpm run check", ["LC_ALL='en_US.UTF-8'"]),
    "env LC_ALL='en_US.UTF-8' sh -c 'cd examples/vite-musea && pnpm run check'",
  );
});

test("task shell commands can forward Vite+ task arguments", () => {
  assert.equal(
    shellCommandForwardingArguments(
      'moon run -q --target native - -- "$@" < tools/moon/scripts/release.mbtx',
      [],
    ),
    "sh -c 'moon run -q --target native - -- \"$@\" < tools/moon/scripts/release.mbtx' --",
  );
});

test("Rust task environments preserve forwarded arguments", () => {
  const command = withRustTaskEnvironment(
    'moon run -q --target native - -- "$@" < tools/moon/scripts/release.mbtx',
    {
      forwardArguments: true,
    },
  );

  assert.match(
    command,
    /sh -c .*moon run -q --target native - -- "\$@" < tools\/moon\/scripts\/release\.mbtx/,
  );
  assert.match(command, / --$/);
});

test("MoonBit task commands prefer the workspace toolchain cache", () => {
  assert.equal(
    moonCommandForEnvironment({}, (candidate) => candidate === ".cache/moonbit/bin/moon"),
    "env MOON_HOME=.cache/moonbit MOON_BIN=.cache/moonbit/bin/moon .cache/moonbit/bin/moon",
  );
});

test("MoonBit task commands preserve the GitHub runner shim", () => {
  assert.equal(
    moonCommandForEnvironment({ MOON_BIN: "/runner-temp/moonbit-shims/moon" }, () => true),
    "/runner-temp/moonbit-shims/moon",
  );
});

test("MoonBit task commands initialize the workspace registry index", () => {
  assert.equal(
    moonRegistryUpdateGuardForEnvironment(
      {},
      (candidate) => candidate === ".cache/moonbit/bin/moon",
    ),
    "( [ -d .cache/moonbit/registry/index/.git ] || env MOON_HOME=.cache/moonbit MOON_BIN=.cache/moonbit/bin/moon .cache/moonbit/bin/moon update )",
  );
});

test("MoonBit task commands leave explicit MoonBit shims untouched", () => {
  assert.equal(
    moonRegistryUpdateGuardForEnvironment(
      { MOON_BIN: "/runner-temp/moonbit-shims/moon" },
      () => true,
    ),
    null,
  );
});

test("release task forwards extra vp run arguments into the MoonBit script", () => {
  const command = (releaseTasks.release as { command: string }).command;

  assert.match(
    command,
    /moon run -q --target native - -- "\$@" < tools\/moon\/scripts\/release\.mbtx/,
  );
  assert.doesNotMatch(command, /env -u MOON_HOME/);
  assert.match(command, / --$/);
});

test("repository JS check enforces the v1 alpha warning budget", () => {
  const command = (checkTasks["check:repo"] as { command: string }).command;

  assert.match(
    command,
    /tools\/vite-plus\/check-warning-budget\.mjs -- \.\/node_modules\/\.bin\/vp check/,
  );
});

test("normalizing a macOS C.UTF-8 environment updates child-process locale variables", () => {
  const env: NodeJS.ProcessEnv = {
    LC_ALL: "C.UTF-8",
    LC_CTYPE: "C.UTF-8",
    LANG: "C.UTF-8",
  };

  normalizeTaskShellLocale("darwin", env);

  assert.equal(env.LC_ALL, "en_US.UTF-8");
  assert.equal(env.LC_CTYPE, "en_US.UTF-8");
  assert.equal(env.LANG, "en_US.UTF-8");
});

test("root build command lookup checks PATH without executing the command", () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-command-lookup-"));
  const binDir = path.join(dir, "bin");
  const sentinelPath = path.join(dir, "sentinel");
  fs.mkdirSync(binDir);

  try {
    writeFakeCommand(
      binDir,
      "side-effect-tool",
      `require("node:fs").writeFileSync(${JSON.stringify(sentinelPath)}, "ran");`,
    );

    assert.equal(commandExists("side-effect-tool", { PATH: binDir }), true);
    assert.equal(fs.existsSync(sentinelPath), false);
    assert.equal(commandExists("missing-tool", { PATH: binDir }), false);
    assert.equal(commandExists(`side-effect-tool; touch ${sentinelPath}`, { PATH: binDir }), false);
    assert.equal(fs.existsSync(sentinelPath), false);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});
