import { readToolingProgram } from "./support/program-path.ts";
import { workspaceTaskSteps } from "../../tools/config/vite-plus/task-plan.ts";
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { commandExists } from "../../tools/config/vite-plus/root-build-task-plugin.ts";
import {
  workspaceTaskEnvironment,
  normalizeTaskShellLocale,
} from "../../tools/config/vite-plus/task-shell.ts";
import { checkTasks } from "../../tools/config/vite-plus/tasks/check.ts";
import { releaseTasks } from "../../tools/config/vite-plus/tasks/release.ts";
import { writeFakeCommand } from "./support/fake-command.ts";

test("task environments preserve the caller's locale and Rust settings", () => {
  const env = {
    LANG: "C.UTF-8",
    VIZE_DARWIN_LIBICONV_LIB: "/nix/iconv/lib",
    LIBRARY_PATH: "/existing",
    RUSTFLAGS: "--cfg caller",
  };
  const mac = workspaceTaskEnvironment("darwin", env);
  assert.deepEqual(mac, {
    ...env,
    LC_ALL: "en_US.UTF-8",
    LC_CTYPE: "en_US.UTF-8",
    LANG: "en_US.UTF-8",
    LIBRARY_PATH: "/nix/iconv/lib:/existing",
    RUSTFLAGS: "-L native=/nix/iconv/lib --cfg caller",
  });
  assert.deepEqual(workspaceTaskEnvironment("darwin", mac), mac);
  assert.deepEqual(workspaceTaskEnvironment("linux", { LANG: "C.UTF-8" }), { LANG: "C.UTF-8" });
  assert.equal(env.LANG, "C.UTF-8");
});

test("the release task forwards arguments to the Release workflow dispatcher", () => {
  assert.deepEqual(workspaceTaskSteps(releaseTasks.release), [
    { program: "node", arguments: ["tools/support/release/release/main.ts", "dispatch"] },
  ]);
});

test("iteration checks enforce the warning budget and the full repo check then checks all TypeScript projects", () => {
  const budget = {
    program: "node",
    arguments: [
      "tools/commands/ci/check-warning-budget.ts",
      "--",
      "./node_modules/.bin/vp",
      "check",
    ],
  };
  assert.deepEqual(workspaceTaskSteps(checkTasks["check:iteration"]), [budget]);
  assert.deepEqual(workspaceTaskSteps(checkTasks["check:repo"]), [
    budget,
    { program: "node", arguments: ["tools/support/typescript/check-project.ts"] },
  ]);
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
      readToolingProgram("commands/task-shell/side-effect-tool.js.in", {
        SENTINEL_PATH: JSON.stringify(sentinelPath),
      }),
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
