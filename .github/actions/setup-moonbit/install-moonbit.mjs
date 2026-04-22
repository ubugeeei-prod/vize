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

const platformMap = {
  Linux: "linux",
  Darwin: "darwin",
  Windows_NT: "windows",
};

const archMap = {
  x64: "x86_64",
  arm64: "aarch64",
};

const platform = platformMap[os.type()];
const arch = archMap[os.arch()];

if (!platform || !arch) {
  console.error(`Unsupported platform for MoonBit: ${os.type()} ${os.arch()}`);
  process.exit(1);
}

function ensureExecutableTree(targetPath) {
  if (!fs.existsSync(targetPath)) {
    return;
  }

  const stats = fs.statSync(targetPath);
  if (stats.isDirectory()) {
    fs.chmodSync(targetPath, 0o755);
    for (const entry of fs.readdirSync(targetPath)) {
      ensureExecutableTree(path.join(targetPath, entry));
    }
    return;
  }

  fs.chmodSync(targetPath, 0o755);
}

const extension = platform === "windows" ? "zip" : "tar.gz";
const moonHome = path.join(runnerTemp, "moonbit");
const archivePath = path.join(runnerTemp, `moonbit-${platform}-${arch}.${extension}`);

fs.rmSync(moonHome, { recursive: true, force: true });
fs.mkdirSync(moonHome, { recursive: true });

const archiveUrl = `https://cli.moonbitlang.com/binaries/latest/moonbit-${platform}-${arch}.${extension}`;

const curlResult = spawnSync("curl", ["-fsSL", archiveUrl, "-o", archivePath], {
  stdio: "inherit",
});
if ((curlResult.status ?? 1) !== 0) {
  process.exit(curlResult.status ?? 1);
}

const extractResult =
  platform === "windows"
    ? spawnSync(
        "powershell",
        [
          "-NoProfile",
          "-Command",
          `Expand-Archive -Path '${archivePath}' -DestinationPath '${moonHome}' -Force`,
        ],
        {
          stdio: "inherit",
        },
      )
    : spawnSync("tar", ["-xzf", archivePath, "-C", moonHome], {
        stdio: "inherit",
      });

if ((extractResult.status ?? 1) !== 0) {
  process.exit(extractResult.status ?? 1);
}

if (platform !== "windows") {
  ensureExecutableTree(path.join(moonHome, "bin"));
}

fs.appendFileSync(githubPath, `${path.join(moonHome, "bin")}\n`);
fs.appendFileSync(githubEnv, `MOON_HOME=${moonHome}\n`);
