import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import process from "node:process";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { fileURLToPath } from "node:url";

// The pinned toolchain lives in one file that `tools/nix/moonbit.nix` also reads, so
// the Nix development shell and CI can never resolve different compilers for
// the same commit. Installing `latest` here is what let the two drift apart.
const moonbitVersionFile = fileURLToPath(new URL("../../../.moonbit-version", import.meta.url));
const moonbitVersion = readPinnedMoonbitVersion(moonbitVersionFile);

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
const mooncExe = path.join(moonBin, os.type() === "Windows_NT" ? "moonc.exe" : "moonc");
const shimDir = path.join(runnerTemp, "moonbit-shims");
const shimMoonCmd = path.join(shimDir, "moon.cmd");
const shimMoonShell = path.join(shimDir, "moon");
const shimMoon = os.type() === "Windows_NT" ? shimMoonCmd : shimMoonShell;
const moonInstallerScript = path.join(runnerTemp, "moonbit-install.ps1");
const moonInstallerUnixScript = path.join(runnerTemp, "moonbit-install.sh");
const moonInstallerSha256 = {
  unix: "46495f8cdc0050f79b6cb195d66478d101cb3601d68506568fbe377fcdf2a9fe",
  windows: "a5101e91ffa9905fb25cd009b9a4aa942971a294bd055c89836e3af89b710c64",
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

function readPinnedMoonbitVersion(filePath) {
  if (!fs.existsSync(filePath)) {
    console.error(`MoonBit version file is required at ${filePath}`);
    console.error("Sparse checkouts that use setup-moonbit must include .moonbit-version.");
    process.exit(1);
  }

  const version = fs.readFileSync(filePath, "utf8").trim();
  if (!version) {
    console.error(`MoonBit version file is empty at ${filePath}`);
    process.exit(1);
  }
  return version;
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

function patchDarwinMoonbitHeader() {
  if (os.type() !== "Darwin") {
    return;
  }

  const moonbitHeader = path.join(moonHome, "include", "moonbit.h");
  const memcpyDeclaration = "void *memcpy(void *dst, const void *src, size_t n);";
  const patchedMemcpyDeclaration = `#ifdef memcpy
#undef memcpy
#endif
${memcpyDeclaration}`;

  if (!fs.existsSync(moonbitHeader)) {
    console.warn(`MoonBit header not found at ${moonbitHeader}; skipping Darwin memcpy patch`);
    return;
  }

  const header = fs.readFileSync(moonbitHeader, "utf8");
  if (header.includes(patchedMemcpyDeclaration)) {
    return;
  }

  if (!header.includes(memcpyDeclaration)) {
    console.warn("MoonBit header memcpy declaration not found; skipping Darwin memcpy patch");
    return;
  }

  fs.writeFileSync(moonbitHeader, header.replace(memcpyDeclaration, patchedMemcpyDeclaration));
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
    input: `import {
  "moonbitlang/async@0.20.1",
  "moonbitlang/async@0.20.1/process",
  "moonbitlang/x@0.4.47/path",
  "moonbitlang/x@0.4.47/sys",
}

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

function installedMoonbitVersion() {
  if (!fs.existsSync(mooncExe)) {
    return undefined;
  }
  const result = spawnSync(mooncExe, ["-v"], {
    encoding: "utf8",
    env: { ...process.env, MOON_HOME: moonHome },
  });
  if ((result.status ?? 1) !== 0) {
    return undefined;
  }
  // `moonc -v` prints `v<version> (<date>)`.
  return (result.stdout ?? "").trim().split(/\s+/)[0]?.replace(/^v/, "");
}

function hasExistingMoonInstall() {
  if (!fs.existsSync(moonExe)) {
    return false;
  }
  const installed = installedMoonbitVersion();
  if (installed === moonbitVersion) {
    return true;
  }
  // A restored cache that predates the pin must never be reused silently:
  // that is exactly how CI kept building against a compiler the flake did
  // not describe. Discard it and install the pinned toolchain instead.
  console.log(`Discarding cached MoonBit ${installed ?? "(unknown)"}; expected ${moonbitVersion}`);
  fs.rmSync(moonHome, { recursive: true, force: true });
  return false;
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
      MOONBIT_INSTALL_VERSION: moonbitVersion,
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
      MOONBIT_INSTALL_VERSION: moonbitVersion,
      SHELL: process.env.SHELL ?? "/bin/bash",
    });
  }

  const installed = installedMoonbitVersion();
  if (installed !== moonbitVersion) {
    console.error(`MoonBit version mismatch: installed ${installed ?? "(unknown)"}`);
    console.error(`Expected ${moonbitVersion} from ${moonbitVersionFile}`);
    process.exit(1);
  }

  run(moonExe, ["update"], {
    ...process.env,
    MOON_HOME: moonHome,
    PATH: `${moonBin}${path.delimiter}${process.env.PATH ?? ""}`,
  });
}

ensureMoonShim();
patchDarwinMoonbitHeader();
smokeTestMoon();

fs.appendFileSync(githubPath, `${shimDir}\n`);
fs.appendFileSync(githubEnv, `MOON_HOME=${moonHome}\n`);
fs.appendFileSync(githubEnv, `MOON_BIN=${shimMoon}\n`);
