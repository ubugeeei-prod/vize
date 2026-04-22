import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import process from "node:process";
import { spawnSync } from "node:child_process";

const runnerTemp = process.env.RUNNER_TEMP;
const githubPath = process.env.GITHUB_PATH;
const githubEnv = process.env.GITHUB_ENV;

if (!runnerTemp || !githubPath || !githubEnv) {
  console.error("RUNNER_TEMP, GITHUB_PATH, and GITHUB_ENV must be set");
  process.exit(1);
}

const moonHome = path.join(runnerTemp, "moonbit");
const moonBin = path.join(moonHome, "bin");
const moonExe = path.join(moonBin, os.type() === "Windows_NT" ? "moon.exe" : "moon");
const shimDir = path.join(runnerTemp, "moonbit-shims");
const shimMoon = path.join(shimDir, os.type() === "Windows_NT" ? "moon.cmd" : "moon");

function run(command, args, env) {
  const result = spawnSync(command, args, {
    stdio: "inherit",
    env,
  });

  if ((result.status ?? 1) !== 0) {
    process.exit(result.status ?? 1);
  }
}

function ensureMoonShim() {
  fs.mkdirSync(shimDir, { recursive: true });
  if (os.type() === "Windows_NT") {
    fs.writeFileSync(
      shimMoon,
      `@echo off\r\nset "MOON_HOME=${moonHome.replaceAll("\\", "\\\\")}"\r\n"${moonExe.replaceAll("\\", "\\\\")}" %*\r\n`,
    );
    return;
  }
  fs.writeFileSync(
    shimMoon,
    `#!/usr/bin/env bash
set -euo pipefail
export MOON_HOME="${moonHome}"
"${moonExe}" "$@"
`,
  );
  fs.chmodSync(shimMoon, 0o755);
}

function smokeTestMoon() {
  const result = spawnSync(moonExe, ["run", "-q", "--target", "native", "-", "--"], {
    stdio: ["pipe", "inherit", "inherit"],
    env: {
      ...process.env,
      MOON_HOME: moonHome,
      PATH: `${shimDir}${path.delimiter}${moonBin}${path.delimiter}${process.env.PATH ?? ""}`,
    },
    input: `///|
import {
  "moonbitlang/async@0.16.8",
  "moonbitlang/x@0.4.41/sys",
}

///|
async fn main {
  println("moonbit-setup-ok")
}
`,
  });
  if ((result.status ?? 1) !== 0) {
    process.exit(result.status ?? 1);
  }
}

if (os.type() === "Windows_NT") {
  run(
    "powershell",
    [
      "-NoProfile",
      "-ExecutionPolicy",
      "Bypass",
      "-Command",
      "Invoke-Expression ((Invoke-WebRequest -UseBasicParsing 'https://cli.moonbitlang.com/install/powershell.ps1').Content)",
    ],
    {
      ...process.env,
      MOON_HOME: moonHome,
    },
  );
} else {
  run("bash", ["-lc", "curl -fsSL https://cli.moonbitlang.com/install/unix.sh | bash"], {
    ...process.env,
    HOME: runnerTemp,
    MOON_HOME: moonHome,
    SHELL: process.env.SHELL ?? "/bin/bash",
  });
}

run(moonExe, ["update"], {
  ...process.env,
  MOON_HOME: moonHome,
  PATH: `${moonBin}${path.delimiter}${process.env.PATH ?? ""}`,
});

ensureMoonShim();
smokeTestMoon();

fs.appendFileSync(githubPath, `${shimDir}\n`);
fs.appendFileSync(githubEnv, `MOON_HOME=${moonHome}\n`);
fs.appendFileSync(githubEnv, `MOON_BIN=${shimMoon}\n`);
