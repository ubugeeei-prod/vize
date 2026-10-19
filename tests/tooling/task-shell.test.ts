import assert from "node:assert/strict";
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
    shellCommandForwardingArguments('node tools/support/release/release/main.ts "$@"', []),
    "sh -c 'node tools/support/release/release/main.ts \"$@\"' --",
  );
});

test("Rust task environments preserve forwarded arguments", () => {
  const command = withRustTaskEnvironment('node tools/support/release/release/main.ts "$@"', {
    forwardArguments: true,
  });

  assert.match(command, /sh -c .*node tools\/support\/release\/release\/main\.ts "\$@"/);
  assert.match(command, / --$/);
});

test("release task dispatches the Release workflow and forwards vp run arguments", () => {
  const command = (releaseTasks.release as { command: string }).command;

  assert.match(command, /node tools\/support\/release\/release\/main\.ts dispatch "\$@"/);
  assert.doesNotMatch(command, /VIZE_RELEASE_REGISTRY_REFRESH|moon/);
  assert.match(command, / --$/);
});

test("repository JS check enforces the v1 alpha warning budget", () => {
  const command = (checkTasks["check:repo"] as { command: string }).command;

  assert.match(
    command,
    /'node' 'tools\/commands\/ci\/check-warning-budget\.ts' '--' '\.\/node_modules\/\.bin\/vp' 'check'/,
  );
});

test("iteration JS checks keep the warning budget without repository verification projects", () => {
  const iteration = (checkTasks["check:iteration"] as { command: string }).command;
  const full = (checkTasks["check:repo"] as { command: string }).command;

  assert.equal(iteration, full.split(" && ")[0]);
  assert.doesNotMatch(iteration, /check-project|rust-script/);
  assert.match(full, /check-project\.ts tsconfig\.public-vrt\.json/);
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
