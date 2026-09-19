import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, readFileSync, realpathSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
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
      writeFileSync(
        path.join(bin, "npx"),
        `#!${process.execPath}\n` +
          `require('node:fs').writeFileSync(process.env.PROBE_OUTPUT, JSON.stringify({args:process.argv.slice(2), cwd:process.cwd(), mode:process.env.NODE_ENV, fixture:process.env.NUXT_TEST_FIXTURES}));\n`,
        { mode: 0o755 },
      );
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
  for (const [program, timeout] of [
    ["process.exit(7)", 5_000],
    ["setInterval(() => {}, 1000)", 50],
  ] as const) {
    assert.throws(() =>
      runAppBuild({
        cwd: os.tmpdir(),
        build: { command: process.execPath, args: ["-e", program], timeout },
      }),
    );
  }
});
