import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { root } from "./support/github-workflows.ts";

function parseGithubEnv(envFile: string): Record<string, string> {
  return Object.fromEntries(
    fs
      .readFileSync(envFile, "utf8")
      .trim()
      .split("\n")
      .map((line) => {
        const index = line.indexOf("=");
        return [line.slice(0, index), line.slice(index + 1)];
      }),
  );
}

test("Zig musl linker wrappers normalize cc-rs target flags", () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-zig-musl-linkers-"));
  const binDir = path.join(tempDir, "bin");
  const envFile = path.join(tempDir, "github-env");
  const fakeZig = path.join(binDir, "zig");
  const scriptPath = path.join(root, "tools", "github", "configure-zig-musl-linkers.sh");

  try {
    fs.mkdirSync(binDir, { recursive: true });
    fs.writeFileSync(fakeZig, '#!/usr/bin/env bash\nprintf \'%s\\n\' "$@" > "$ZIG_ARGS_LOG"\n');
    fs.chmodSync(fakeZig, 0o755);

    const pathEnv = `${binDir}${path.delimiter}${process.env.PATH ?? ""}`;
    execFileSync("bash", [scriptPath], {
      env: { ...process.env, GITHUB_ENV: envFile, PATH: pathEnv, RUNNER_TEMP: tempDir },
    });

    const githubEnv = parseGithubEnv(envFile);
    const runWrapper = (envName: string, args: string[]) => {
      const logPath = path.join(tempDir, `${envName}.log`);
      execFileSync(githubEnv[envName], args, {
        env: { ...process.env, PATH: pathEnv, ZIG_ARGS_LOG: logPath },
      });
      return fs.readFileSync(logPath, "utf8").trim().split("\n");
    };

    assert.deepEqual(
      runWrapper("CC_x86_64_unknown_linux_musl", [
        "--target=x86_64-unknown-linux-musl",
        "-O3",
        "-c",
        "static.c",
      ]),
      ["cc", "-target", "x86_64-linux-musl", "-O3", "-c", "static.c"],
    );
    assert.deepEqual(
      runWrapper("CC_aarch64_unknown_linux_musl", [
        "--target",
        "aarch64-unknown-linux-musl",
        "-DMI_DEBUG=0",
      ]),
      ["cc", "-target", "aarch64-linux-musl", "-DMI_DEBUG=0"],
    );
  } finally {
    fs.rmSync(tempDir, { recursive: true, force: true });
  }
});
