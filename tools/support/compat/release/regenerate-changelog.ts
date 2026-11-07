// Regenerates the root CHANGELOG.md from the repository git history.
//
// Uses `git-cliff` configured by `cliff.toml`. If git-cliff is not on PATH,
// this script downloads the pinned release for the current platform into a
// scratch directory and runs it from there, so the regeneration is
// reproducible on contributor machines and in CI without a global install.

import { execFile, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { createWriteStream, existsSync, mkdtempSync, readFileSync, statSync } from "node:fs";
import { chmod, mkdir } from "node:fs/promises";
import https from "node:https";
import { arch as osArch, platform as osPlatform, tmpdir } from "node:os";
import path from "node:path";
import { Transform } from "node:stream";
import { pipeline } from "node:stream/promises";
import { fileURLToPath } from "node:url";
import { promisify } from "node:util";

const execFileP = promisify(execFile);

const GIT_CLIFF_VERSION = "2.6.1";
const MAX_GIT_CLIFF_ARCHIVE_BYTES = 16 * 1024 * 1024;
const scriptPath = fileURLToPath(import.meta.url);
const __dirname = path.dirname(scriptPath);
const repoRoot = path.resolve(__dirname, "../../../..");

export interface GitCliffAsset {
  readonly name: string;
  readonly sha512: string;
}

export function resolveGitCliffAsset(platform = osPlatform(), arch = osArch()): GitCliffAsset {
  if (platform === "linux" && arch === "x64") {
    return {
      name: `git-cliff-${GIT_CLIFF_VERSION}-x86_64-unknown-linux-gnu.tar.gz`,
      sha512:
        "b1e7a6b70fc794accdccd9717eff5a8bf8b507f0308da44d837d935be31b18b19db3758d1a9bb4975fbca7d01ddaae251229a753ae2cd2ecc24845cfc50d067f",
    };
  }
  if (platform === "linux" && arch === "arm64") {
    return {
      name: `git-cliff-${GIT_CLIFF_VERSION}-aarch64-unknown-linux-gnu.tar.gz`,
      sha512:
        "6e8aafddd1e1d163290561cd595fa86971b034012b0602fd8f554f831158af56d3c8f9d82793b4e79d1300f6620091602d81b49110f96ca87068d5df6127cf57",
    };
  }
  if (platform === "darwin" && arch === "arm64") {
    return {
      name: `git-cliff-${GIT_CLIFF_VERSION}-aarch64-apple-darwin.tar.gz`,
      sha512:
        "d507021de298ceba43fd5cc1594c9b868888b5f8a685bc6910d31dc0902eb7b09e25de06a6ef20fb9fc98a505127e695e647747bc97a3697d626d54d6992b394",
    };
  }
  if (platform === "darwin" && arch === "x64") {
    return {
      name: `git-cliff-${GIT_CLIFF_VERSION}-x86_64-apple-darwin.tar.gz`,
      sha512:
        "acc7e28da2af6337fe67b05003d5468cba4fc3014b2d51e7cf9f7f01e8ee11d4bc10845ba282694320648e7627af67bfe62d00a7e7b6f9faec8b2c38fbbd00a3",
    };
  }
  if (platform === "win32" && arch === "x64") {
    return {
      name: `git-cliff-${GIT_CLIFF_VERSION}-x86_64-pc-windows-msvc.zip`,
      sha512:
        "9da4c3bb4c407ded67e411544c4d4b0c2b748c20db5181b9da72aeb652efc8c939a0e07411886d4132d49581c02e8b7ecae1d127dcc4508159a2f70fea780e76",
    };
  }
  throw new Error(`Unsupported platform/arch for git-cliff fallback: ${platform}/${arch}`);
}

function hasGitCliffOnPath(): boolean {
  const probe = spawnSync("git-cliff", ["--version"], { stdio: "ignore" });
  return probe.status === 0;
}

async function downloadFile(url: string, dest: string, redirects = 0): Promise<void> {
  await new Promise<void>((resolve, reject) => {
    const request = https.get(url, (response) => {
      if (
        (response.statusCode === 301 ||
          response.statusCode === 302 ||
          response.statusCode === 307 ||
          response.statusCode === 308) &&
        response.headers.location
      ) {
        response.resume();
        if (redirects >= 5) {
          reject(new Error(`Too many redirects for ${url}`));
          return;
        }
        const redirect = new URL(response.headers.location, url);
        if (redirect.protocol !== "https:") {
          reject(new Error(`Refusing non-HTTPS redirect for ${url}`));
          return;
        }
        downloadFile(redirect.href, dest, redirects + 1).then(resolve, reject);
        return;
      }
      if (response.statusCode !== 200) {
        response.resume();
        reject(new Error(`HTTP ${response.statusCode} for ${url}`));
        return;
      }
      const contentLength = Number(response.headers["content-length"]);
      if (Number.isFinite(contentLength) && contentLength > MAX_GIT_CLIFF_ARCHIVE_BYTES) {
        response.resume();
        reject(new Error(`Downloaded git-cliff archive exceeds the size limit`));
        return;
      }
      let received = 0;
      const sizeLimit = new Transform({
        transform(chunk, _encoding, callback) {
          received += chunk.length;
          if (received > MAX_GIT_CLIFF_ARCHIVE_BYTES) {
            callback(new Error(`Downloaded git-cliff archive exceeds the size limit`));
            return;
          }
          callback(null, chunk);
        },
      });
      pipeline(response, sizeLimit, createWriteStream(dest)).then(resolve, reject);
    });
    request.on("error", reject);
  });
}

async function ensureGitCliff(): Promise<string> {
  if (hasGitCliffOnPath()) return "git-cliff";

  const asset = resolveGitCliffAsset();
  const url = `https://github.com/orhun/git-cliff/releases/download/v${GIT_CLIFF_VERSION}/${asset.name}`;
  const scratch = mkdtempSync(path.join(tmpdir(), "vize-git-cliff-"));
  const archive = path.join(scratch, asset.name);
  await downloadFile(url, archive);
  verifyGitCliffArchive(archive, asset.sha512);

  if (asset.name.endsWith(".tar.gz")) {
    await execFileP("tar", ["-xzf", archive, "-C", scratch]);
  } else {
    await execFileP("unzip", ["-q", archive, "-d", scratch]);
  }

  const expected = asset.name.replace(/\.tar\.gz$|\.zip$/, "");
  const bin = path.join(
    scratch,
    expected,
    osPlatform() === "win32" ? "git-cliff.exe" : "git-cliff",
  );
  await chmod(bin, 0o755);
  return bin;
}

export function verifyGitCliffArchive(file: string, expectedSha512: string): void {
  const stat = statSync(file);
  if (!stat.isFile() || stat.size > MAX_GIT_CLIFF_ARCHIVE_BYTES) {
    throw new Error(
      `Downloaded git-cliff archive is not a regular file within the ${MAX_GIT_CLIFF_ARCHIVE_BYTES}-byte size limit`,
    );
  }
  const actual = createHash("sha512").update(readFileSync(file)).digest("hex");
  if (actual !== expectedSha512) throw new Error(`SHA-512 mismatch for ${file}`);
}

export function buildGitCliffArgs(argv: readonly string[]): string[] {
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

async function main(): Promise<void> {
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
