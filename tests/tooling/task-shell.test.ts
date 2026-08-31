import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { commandExists } from "../../tools/config/vite-plus/root-build-task-plugin.ts";
import {
  getTaskShellLocaleAssignments,
  normalizeTaskShellLocale,
  shellCommand,
  shellCommandForwardingArguments,
  withRustTaskEnvironment,
} from "../../tools/config/vite-plus/task-shell.ts";
import {
  localVp,
  moonCommandForEnvironment,
  moonRegistryRefreshCommandForEnvironment,
  moonRegistryUpdateGuardForEnvironment,
} from "../../tools/config/vite-plus/task-commands.ts";
import { checkTasks } from "../../tools/config/vite-plus/tasks/check.ts";
import { releaseTasks } from "../../tools/config/vite-plus/tasks/release.ts";
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
      'moon run -q --target native tools/moon/cmd/release -- "$@"',
      [],
    ),
    "sh -c 'moon run -q --target native tools/moon/cmd/release -- \"$@\"' --",
  );
});

test("Rust task environments preserve forwarded arguments", () => {
  const command = withRustTaskEnvironment(
    'moon run -q --target native tools/moon/cmd/release -- "$@"',
    {
      forwardArguments: true,
    },
  );

  assert.match(command, /sh -c .*moon run -q --target native tools\/moon\/cmd\/release -- "\$@"/);
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

test("release registry refreshes use an explicit MoonBit shim", () => {
  assert.equal(
    moonRegistryRefreshCommandForEnvironment(
      { MOON_BIN: "/runner-temp/moonbit-shims/moon" },
      () => true,
    ),
    "/runner-temp/moonbit-shims/moon update",
  );
});

test("release task refreshes the MoonBit registry and forwards vp run arguments", () => {
  const command = (releaseTasks.release as { command: string }).command;

  assert.match(
    command,
    /moon update && .*moon run -q --target native tools\/moon\/cmd\/release -- "\$@"/,
  );
  assert.match(command, /moon run -q --target native tools\/moon\/cmd\/release -- "\$@"/);
  assert.doesNotMatch(command, /env -u MOON_HOME/);
  assert.match(command, / --$/);
});

function gitRepositoryState() {
  const read = (...args: string[]) => {
    const result = spawnSync("git", args, { encoding: "utf8" });
    assert.equal(result.status, 0, result.stderr);
    return result.stdout;
  };
  return {
    head: read("rev-parse", "HEAD"),
    status: read("status", "--porcelain=v1", "--untracked-files=all"),
    tags: read("tag", "--list"),
  };
}

test(
  "release task prompts through a controlling terminal and aborts without mutation",
  { skip: process.platform === "win32" },
  () => {
    const before = gitRepositoryState();
    const result = spawnSync(
      "python3",
      [
        "tests/tooling/support/pty-command.py",
        "Proceed with release? [y/N]",
        "n\n",
        localVp,
        "run",
        "release",
        "minor",
      ],
      { encoding: "utf8", timeout: 30_000 },
    );
    const output = result.stdout + result.stderr;

    assert.equal(result.error, undefined, output);
    assert.equal(result.status, 1, output);
    assert.match(output, /Proceed with release\? \[y\/N\]/);
    assert.match(output, /Aborted\./);
    assert.deepEqual(gitRepositoryState(), before);
  },
);

test("repository JS check enforces the v1 alpha warning budget", () => {
  const command = (checkTasks["check:repo"] as { command: string }).command;

  assert.match(
    command,
    /'rust-script' 'tools\/commands\/ci\/check-warning-budget\.rs' '--' '\.\/node_modules\/\.bin\/vp' 'check'/,
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
