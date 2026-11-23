import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, readFileSync, realpathSync, rmSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { installToolingProgram, toolingProgram } from "./support/program-path.ts";
import { runAppBuild } from "../_helpers/app-build.ts";

test(
  "preview preserves the fixture runner, environment and argument boundaries",
  {
    skip: process.platform === "win32" ? "POSIX executable fixture" : false,
  },
  () => {
    const root = mkdtempSync(path.join(os.tmpdir(), "vize preview "));
    try {
      const bin = path.join(root, "bin");
      mkdirSync(bin);
      const result = path.join(root, "result.json");
      installToolingProgram(path.join(bin, "npx"), "workflow/record-preview-build.cjs");
      const args = ["-y", "pnpm@10", "build", "--filter", "path with spaces", "$(must-not-run)"];
      runAppBuild({
        cwd: root,
        env: { PATH: bin, PROBE_OUTPUT: result, NUXT_TEST_FIXTURES: "true" },
        build: { command: "npx", args, timeout: 5_000 },
      });
      assert.deepEqual(JSON.parse(readFileSync(result, "utf8")), {
        args,
        cwd: realpathSync(root),
        mode: "production",
        fixture: "true",
      });
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  },
);

test("preview propagates nonzero exits and build timeouts", () => {
  for (const [args, timeout] of [
    [[toolingProgram("process/exit-with-status.js"), "7"], 5_000],
    [[toolingProgram("process/hold-process-open.js")], 50],
  ] as const) {
    assert.throws(() =>
      runAppBuild({
        cwd: os.tmpdir(),
        build: { command: process.execPath, args: [...args], timeout },
      }),
    );
  }
});
