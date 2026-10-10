import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import process from "node:process";
import { spawn, spawnSync } from "node:child_process";
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

async function updateRegistry(command, args, env) {
  for (let attempt = 1; attempt <= 3; attempt++) {
    const stderr = [];
    const stderrWrites = [];
    const result = await new Promise((resolve) => {
      let error;
      const child = spawn(command, args, { stdio: ["inherit", "inherit", "pipe"], env });
      child.stderr?.on("data", (chunk) => {
        // Keep every attempt's original bytes in the Actions log while retaining
        // stderr only to recognize the specific registry Git transport failure.
        stderrWrites.push(new Promise((resolve) => process.stderr.write(chunk, resolve)));
        stderr.push(chunk);
      });
      child.once("error", (cause) => {
        error = cause;
        console.error(`MoonBit registry update failed to start: ${cause.message}`);
      });
      child.once("close", (status, signal) => resolve({ status, signal, error }));
    });
    // A closed child does not imply that the parent's piped stderr is drained.
    await Promise.all(stderrWrites);
    if (result.status === 0) {
      return;
    }

    const text = Buffer.concat(stderr).toString("utf8");
    // This private helper is used only by the validated cold-install moon update.
    // RPC errors omit the URL; their complete Moon/Git failure profile is bound
    // to that same pinned executable and registry operation by the sole caller.
    const registryGitFailure =
      /^Error: update failed\s*$/m.test(text) &&
      /^\s*\d+: failed to (?:clone|fetch) registry index\s*$/m.test(text) &&
      /^\s*\d+: non-zero exit code: exit status: 128\s*$/m.test(text) &&
      /^\s*git stderr:\s*$/m.test(text);
    const exactUrlFailure =
      /^\s*fatal: unable to access 'https:\/\/mooncakes\.io\/git\/index\/': The requested URL returned error: (?:502|503|504)\s*$/m.test(
        text,
      );
    const registryRpcFailure =
      result.status === 255 &&
      /^\s*error: RPC failed; HTTP (502|503|504) curl 22 The requested URL returned error: \1\s*$/m.test(
        text,
      ) &&
      /^\s*fatal: expected 'packfile'\s*$/m.test(text) &&
      text.split(/\r?\n/).some((line) => {
        const destination = path.join(env.MOON_HOME, "registry", "index");
        return (
          line.trim() === `Cloning into '${destination}'...` ||
          line.trim() === `Cloning into '${destination.replaceAll("\\", "/")}'...`
        );
      });
    const transientTransport =
      registryGitFailure &&
      (exactUrlFailure || registryRpcFailure) &&
      !/authentication|could not read username|permission denied|unauthorized|forbidden|certificate|validation|invalid|hash mismatch|version mismatch|compil|failed to (?:verify|validate)/i.test(
        text,
      ) &&
      !/The requested URL returned error: (?!(?:502|503|504)\b)\d+/.test(text) &&
      !/fatal: unable to access '(?!https:\/\/mooncakes\.io\/git\/index\/')[^']+'/.test(text);
    if (result.error || result.signal || !transientTransport || attempt === 3) {
      process.exit(result.error ? 1 : (result.status ?? 1));
    }

    const delay = attempt * 1000;
    console.warn(
      `MoonBit registry update transport failed (attempt ${attempt}/3, exit ${result.status}); retrying in ${delay}ms`,
    );
    await new Promise((resolve) => setTimeout(resolve, delay));
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

  await updateRegistry(moonExe, ["update"], {
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
