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
const shimMoonCmd = path.join(shimDir, "moon.cmd");
const shimMoonShell = path.join(shimDir, "moon");
const shimMoon = os.type() === "Windows_NT" ? shimMoonCmd : shimMoonShell;
const moonInstallerScript = path.join(runnerTemp, "moonbit-install.ps1");

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
      shimMoonCmd,
      `@echo off\r\nset "MOON_HOME=${moonHome.replaceAll("\\", "\\\\")}"\r\n"${moonExe.replaceAll("\\", "\\\\")}" %*\r\n`,
    );
    fs.writeFileSync(
      shimMoonShell,
      `#!/usr/bin/env bash
set -euo pipefail
export MOON_HOME="${moonHome.replaceAll("\\", "/")}"
"${moonExe.replaceAll("\\", "/")}" "$@"
`,
    );
    fs.chmodSync(shimMoonShell, 0o755);
    return;
  }
  fs.writeFileSync(
    shimMoonShell,
    `#!/usr/bin/env bash
set -euo pipefail
export MOON_HOME="${moonHome}"
"${moonExe}" "$@"
`,
  );
  fs.chmodSync(shimMoonShell, 0o755);
}

function smokeTestMoon() {
  const smokeTestCommand =
    os.type() === "Windows_NT"
      ? { command: "cmd", args: ["/C", "echo", "moonbit-setup-ok"] }
      : { command: "sh", args: ["-lc", "printf moonbit-setup-ok"] };

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
  "moonbitlang/async@0.16.8/process",
  "moonbitlang/x@0.4.41/sys",
}

///|
async fn main {
  let exit_code = @process.run(${JSON.stringify(smokeTestCommand.command)}, [${smokeTestCommand.args
    .map((arg) => JSON.stringify(arg))
    .join(", ")}])
  if exit_code != 0 {
    @sys.exit(exit_code)
  }
}
`,
  });
  if ((result.status ?? 1) !== 0) {
    process.exit(result.status ?? 1);
  }
}

function hasExistingMoonInstall() {
  return fs.existsSync(moonExe);
}

if (!hasExistingMoonInstall()) {
  if (os.type() === "Windows_NT") {
    run(
      "pwsh",
      [
        "-NoProfile",
        "-ExecutionPolicy",
        "Bypass",
        "-Command",
        `Invoke-WebRequest -UseBasicParsing 'https://cli.moonbitlang.com/install/powershell.ps1' -OutFile "${moonInstallerScript}"`,
      ],
      {
        ...process.env,
        MOON_HOME: moonHome,
      },
    );
    run("pwsh", ["-NoProfile", "-ExecutionPolicy", "Bypass", "-File", moonInstallerScript], {
      ...process.env,
      MOON_HOME: moonHome,
    });
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
}

ensureMoonShim();
smokeTestMoon();

fs.appendFileSync(githubPath, `${shimDir}\n`);
fs.appendFileSync(githubEnv, `MOON_HOME=${moonHome}\n`);
fs.appendFileSync(githubEnv, `MOON_BIN=${shimMoon}\n`);
