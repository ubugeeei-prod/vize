import assert from "node:assert/strict";
import { test } from "node:test";

import {
  getTaskShellLocaleAssignments,
  normalizeTaskShellLocale,
  shellCommand,
  shellCommandForwardingArguments,
  withRustTaskEnvironment,
} from "../../tools/vite-plus/task-shell.ts";
import { moonCommandForEnvironment } from "../../tools/vite-plus/task-commands.ts";
import { releaseTasks } from "../../tools/vite-plus/tasks/release.ts";

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

test("release task forwards extra vp run arguments into the MoonBit script", () => {
  const command = (releaseTasks.release as { command: string }).command;

  assert.match(
    command,
    /moon run -q --target native - -- "\$@" < tools\/moon\/scripts\/release\.mbtx/,
  );
  assert.doesNotMatch(command, /env -u MOON_HOME/);
  assert.match(command, / --$/);
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
