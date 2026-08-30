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

test("Zig musl wrappers use rust-lld for final links and normalize cc-rs flags", () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-zig-musl-linkers-"));
  const binDir = path.join(tempDir, "bin");
  const envFile = path.join(tempDir, "github-env");
  const fakeZig = path.join(binDir, "zig");
  const commandPath = path.join(
    root,
    "tools",
    "commands",
    "ci",
    "github",
    "configure-zig-musl-linkers.rs",
  );

  try {
    fs.mkdirSync(binDir, { recursive: true });
    fs.writeFileSync(fakeZig, '#!/usr/bin/env bash\nprintf \'%s\\n\' "$@" > "$ZIG_ARGS_LOG"\n');
    fs.chmodSync(fakeZig, 0o755);

    const pathEnv = `${binDir}${path.delimiter}${process.env.PATH ?? ""}`;
    execFileSync("rust-script", [commandPath], {
      cwd: root,
      env: { ...process.env, GITHUB_ENV: envFile, PATH: pathEnv, RUNNER_TEMP: tempDir },
    });

    const githubEnv = parseGithubEnv(envFile);
    const runCcWrapper = (envName: string, args: string[]) => {
      const logPath = path.join(tempDir, `${envName}.log`);
      execFileSync(githubEnv[envName], args, {
        env: { ...process.env, PATH: pathEnv, ZIG_ARGS_LOG: logPath },
      });
      return fs.readFileSync(logPath, "utf8").trim().split("\n");
    };

    assert.equal(githubEnv.CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER, "rust-lld");
    assert.equal(githubEnv.CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER, "rust-lld");
    assert.deepEqual(
      runCcWrapper("CC_x86_64_unknown_linux_musl", [
        "--target=x86_64-unknown-linux-musl",
        "-O3",
        "-c",
        "static.c",
      ]),
      ["cc", "-target", "x86_64-linux-musl", "-O3", "-c", "static.c"],
    );
    assert.deepEqual(
      runCcWrapper("CC_aarch64_unknown_linux_musl", [
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

test("musl CLI verifier rejects dynamic interpreters and glibc symbol requirements", () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-musl-verifier-"));
  const binDir = path.join(tempDir, "bin");
  const target = "x86_64-unknown-linux-musl";
  const binary = path.join(tempDir, "target", target, "release", "vize");
  const commandPath = path.join(
    root,
    "tools",
    "commands",
    "ci",
    "github",
    "verify-musl-cli-binary.rs",
  );

  try {
    fs.mkdirSync(path.dirname(binary), { recursive: true });
    fs.mkdirSync(binDir, { recursive: true });
    fs.writeFileSync(binary, "");
    fs.writeFileSync(
      path.join(binDir, "file"),
      "#!/usr/bin/env bash\nprintf 'ELF static-pie\\n'\n",
    );
    fs.writeFileSync(
      path.join(binDir, "readelf"),
      [
        "#!/usr/bin/env bash",
        'if [[ "${VERIFY_MODE:-ok}" == "interpreter" ]]; then',
        "  printf '[Requesting program interpreter: /lib64/ld-linux-x86-64.so.2]\\n'",
        "else",
        "  printf 'Program Headers:\\n'",
        "fi",
        "",
      ].join("\n"),
    );
    fs.writeFileSync(
      path.join(binDir, "strings"),
      [
        "#!/usr/bin/env bash",
        'if [[ "${VERIFY_MODE:-ok}" == "glibc" ]]; then',
        "  printf 'GLIBC_2.39\\n'",
        "else",
        "  printf 'musl\\n'",
        "fi",
        "",
      ].join("\n"),
    );
    for (const command of ["file", "readelf", "strings"]) {
      fs.chmodSync(path.join(binDir, command), 0o755);
    }

    const runVerifier = (mode: string) =>
      execFileSync("rust-script", [commandPath, target], {
        cwd: tempDir,
        env: {
          ...process.env,
          PATH: `${binDir}${path.delimiter}${process.env.PATH}`,
          VERIFY_MODE: mode,
        },
      });

    runVerifier("ok");
    assert.throws(() => runVerifier("interpreter"));
    assert.throws(() => runVerifier("glibc"));
  } finally {
    fs.rmSync(tempDir, { recursive: true, force: true });
  }
});
