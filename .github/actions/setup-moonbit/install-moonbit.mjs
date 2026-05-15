import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import process from "node:process";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";

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
const moonInstallerUnixScript = path.join(runnerTemp, "moonbit-install.sh");
const moonInstallerSha256 = {
  unix: "802ea2310c13e2a6b447050a2c82e60b4d5f222ec7c6a8e55b77df09d4b4da7d",
  windows: "e1b22bd41363ca8cdb1480b523a8c61ce57ffcb4a581375369ff16c5afd5c5b7",
};

function run(command, args, env) {
  const result = spawnSync(command, args, {
    stdio: "inherit",
    env,
  });

  if ((result.status ?? 1) !== 0) {
    process.exit(result.status ?? 1);
  }
}

function sha256File(filePath) {
  return createHash("sha256").update(fs.readFileSync(filePath)).digest("hex");
}

function verifyInstaller(filePath, expectedHash) {
  const actualHash = sha256File(filePath);
  if (actualHash !== expectedHash) {
    console.error(`MoonBit installer hash mismatch for ${filePath}`);
    console.error(`Expected: ${expectedHash}`);
    console.error(`Actual:   ${actualHash}`);
    process.exit(1);
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
    verifyInstaller(moonInstallerScript, moonInstallerSha256.windows);
    run("pwsh", ["-NoProfile", "-ExecutionPolicy", "Bypass", "-File", moonInstallerScript], {
      ...process.env,
      MOON_HOME: moonHome,
    });
  } else {
    run(
      "curl",
      ["-fsSL", "https://cli.moonbitlang.com/install/unix.sh", "-o", moonInstallerUnixScript],
      {
        ...process.env,
        HOME: runnerTemp,
        MOON_HOME: moonHome,
        SHELL: process.env.SHELL ?? "/bin/bash",
      },
    );
    verifyInstaller(moonInstallerUnixScript, moonInstallerSha256.unix);
    run("bash", [moonInstallerUnixScript], {
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
