#!/usr/bin/env node
// Regenerates the root CHANGELOG.md from the repository git history.
//
// Uses `git-cliff` configured by `cliff.toml`. If git-cliff is not on PATH,
// this script downloads the pinned release for the current platform into a
// scratch directory and runs it from there, so the regeneration is
// reproducible on contributor machines and in CI without a global install.

import { spawnSync } from "node:child_process";
import { existsSync, mkdtempSync } from "node:fs";
import { mkdir, chmod } from "node:fs/promises";
import { tmpdir, arch as osArch, platform as osPlatform } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import https from "node:https";
import { createWriteStream } from "node:fs";
import { pipeline } from "node:stream/promises";
import { execFile } from "node:child_process";
import { promisify } from "node:util";

const execFileP = promisify(execFile);

const GIT_CLIFF_VERSION = "2.6.1";
const scriptPath = fileURLToPath(import.meta.url);
const __dirname = path.dirname(scriptPath);
const repoRoot = path.resolve(__dirname, "../..");

function resolveAsset() {
  const platform = osPlatform();
  const arch = osArch();
  if (platform === "linux" && arch === "x64") {
    return `git-cliff-${GIT_CLIFF_VERSION}-x86_64-unknown-linux-gnu.tar.gz`;
  }
  if (platform === "linux" && arch === "arm64") {
    return `git-cliff-${GIT_CLIFF_VERSION}-aarch64-unknown-linux-gnu.tar.gz`;
  }
  if (platform === "darwin" && arch === "arm64") {
    return `git-cliff-${GIT_CLIFF_VERSION}-aarch64-apple-darwin.tar.gz`;
  }
  if (platform === "darwin" && arch === "x64") {
    return `git-cliff-${GIT_CLIFF_VERSION}-x86_64-apple-darwin.tar.gz`;
  }
  if (platform === "win32" && arch === "x64") {
    return `git-cliff-${GIT_CLIFF_VERSION}-x86_64-pc-windows-msvc.zip`;
  }
  throw new Error(`Unsupported platform/arch for git-cliff fallback: ${platform}/${arch}`);
}

function hasGitCliffOnPath() {
  const probe = spawnSync("git-cliff", ["--version"], { stdio: "ignore" });
  return probe.status === 0;
}

async function downloadFile(url, dest) {
  await new Promise((resolve, reject) => {
    const request = https.get(url, (response) => {
      if (
        (response.statusCode === 301 ||
          response.statusCode === 302 ||
          response.statusCode === 307 ||
          response.statusCode === 308) &&
        response.headers.location
      ) {
        downloadFile(response.headers.location, dest).then(resolve, reject);
        return;
      }
      if (response.statusCode !== 200) {
        reject(new Error(`HTTP ${response.statusCode} for ${url}`));
        return;
      }
      pipeline(response, createWriteStream(dest)).then(resolve, reject);
    });
    request.on("error", reject);
  });
}

async function ensureGitCliff() {
  if (hasGitCliffOnPath()) return "git-cliff";

  const asset = resolveAsset();
  const url = `https://github.com/orhun/git-cliff/releases/download/v${GIT_CLIFF_VERSION}/${asset}`;
  const scratch = mkdtempSync(path.join(tmpdir(), "vize-git-cliff-"));
  const archive = path.join(scratch, asset);
  await downloadFile(url, archive);

  if (asset.endsWith(".tar.gz")) {
    await execFileP("tar", ["-xzf", archive, "-C", scratch]);
  } else {
    await execFileP("unzip", ["-q", archive, "-d", scratch]);
  }

  const expected = asset.replace(/\.tar\.gz$|\.zip$/, "");
  const bin = path.join(
    scratch,
    expected,
    osPlatform() === "win32" ? "git-cliff.exe" : "git-cliff",
  );
  await chmod(bin, 0o755);
  return bin;
}

export function buildGitCliffArgs(argv) {
  const args = ["--config", "cliff.toml", "--output", "CHANGELOG.md"];

  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i];
    if (arg === "--unreleased") {
      args.push("--unreleased");
      continue;
    }
    if (arg === "--latest") {
      args.push("--latest");
      continue;
    }
    if (arg === "--tag") {
      const tag = argv[i + 1];
      if (tag == null || tag === "" || tag.startsWith("--")) {
        throw new Error("Missing value for --tag");
      }
      args.push("--tag", tag);
      i += 1;
      continue;
    }
    throw new Error(`Unknown argument: ${arg}`);
  }

  return args;
}

async function main() {
  const args = buildGitCliffArgs(process.argv.slice(2));
  await mkdir(path.dirname(path.join(repoRoot, "CHANGELOG.md")), { recursive: true });
  const cliffBin = await ensureGitCliff();

  const result = spawnSync(cliffBin, args, { cwd: repoRoot, stdio: "inherit" });
  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }

  if (!existsSync(path.join(repoRoot, "CHANGELOG.md"))) {
    console.error("git-cliff finished but CHANGELOG.md is missing.");
    process.exit(1);
  }
}

if (process.argv[1] != null && path.resolve(process.argv[1]) === scriptPath) {
  main().catch((error) => {
    console.error(error);
    process.exit(1);
  });
}
