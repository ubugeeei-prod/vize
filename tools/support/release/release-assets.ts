import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { createReadStream } from "node:fs";
import {
  appendFileSync,
  lstatSync,
  mkdtempSync,
  readdirSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { pathToFileURL } from "node:url";

type Identity = { sourceSha: string; runId: string; attempt: number; tag: string };
const receiptName = "release-assets.json";

export function releaseIdentity(env: NodeJS.ProcessEnv, checkout: string): Identity {
  const sourceSha = env.RELEASE_SOURCE_SHA ?? "";
  const runId = env.GITHUB_RUN_ID ?? "";
  const attempt = Number(env.GITHUB_RUN_ATTEMPT);
  const tag = env.RELEASE_TAG_NAME ?? "";
  if (
    !/^[0-9a-f]{40}$/.test(sourceSha) ||
    sourceSha !== env.GITHUB_SHA ||
    sourceSha !== checkout ||
    !/^[1-9]\d*$/.test(runId) ||
    !Number.isSafeInteger(attempt) ||
    attempt < 1 ||
    !/^v\d+\.\d+\.\d+(?:-[\w.-]+)?$/.test(tag)
  ) {
    throw new Error("Release assets require the exact checked-out source, run, attempt and tag");
  }
  return { sourceSha, runId, attempt, tag };
}

function sbomFiles(tag: string): string[] {
  return ["cyclonedx", "spdx"].flatMap((format) =>
    ["", ".sig", ".pem", ".cosign.bundle"].map(
      (suffix) => `vize-${tag}-${format}.sbom.json${suffix}`,
    ),
  );
}

function regularPayload(path: string): void {
  const stat = lstatSync(path);
  if (stat.isFile()) return;
  if (!stat.isDirectory()) throw new Error(`Release asset is not a regular file: ${path}`);
  for (const name of readdirSync(path)) regularPayload(join(path, name));
}

async function digest(path: string): Promise<string> {
  const hash = createHash("sha256");
  for await (const chunk of createReadStream(path)) hash.update(chunk);
  return hash.digest("hex");
}

/** Seal the already signed candidate assets; never rebuild or repack product binaries. */
export async function packReleaseAssets(
  root: string,
  archive: string,
  identity: Identity,
): Promise<string> {
  const payload = ["cli-artifacts", "editor-extension-artifacts", ...sbomFiles(identity.tag)];
  for (const path of payload) regularPayload(join(root, path));
  const temporary = mkdtempSync(join(tmpdir(), "vize-release-assets-"));
  try {
    writeFileSync(join(temporary, receiptName), `${JSON.stringify(identity)}\n`);
    execFileSync("tar", ["-cf", archive, ...payload, "-C", temporary, receiptName], {
      cwd: root,
      stdio: "pipe",
    });
    return await digest(archive);
  } finally {
    rmSync(temporary, { recursive: true, force: true });
  }
}

/** A failed-job rerun may consume the same run's earlier successful preparation. */
export async function extractReleaseAssets(
  root: string,
  archive: string,
  expectedDigest: string,
  identity: Identity,
): Promise<void> {
  if (!/^[0-9a-f]{64}$/.test(expectedDigest) || (await digest(archive)) !== expectedDigest) {
    throw new Error("Prepared release asset digest does not match its producing job");
  }
  const entries = execFileSync("tar", ["-tf", archive], { encoding: "utf8" }).trim().split("\n");
  const allowed = new Set([receiptName, ...sbomFiles(identity.tag)]);
  if (
    entries.some(
      (path) =>
        path.split("/").includes("..") ||
        !(allowed.has(path) || /^(?:cli-artifacts|editor-extension-artifacts)\//.test(path)),
    )
  ) {
    throw new Error("Prepared release assets contain an unexpected path");
  }
  const receipt = JSON.parse(
    execFileSync("tar", ["-xOf", archive, receiptName], { encoding: "utf8" }),
  ) as Identity;
  if (
    receipt.sourceSha !== identity.sourceSha ||
    receipt.runId !== identity.runId ||
    receipt.tag !== identity.tag ||
    !Number.isSafeInteger(receipt.attempt) ||
    receipt.attempt < 1 ||
    receipt.attempt > identity.attempt
  ) {
    throw new Error("Prepared release assets belong to a different source, run, tag or attempt");
  }
  execFileSync("tar", ["-xf", archive, "--no-same-owner"], { cwd: root, stdio: "pipe" });
}

async function main(): Promise<void> {
  const [operation, archive, expectedDigest] = process.argv.slice(2);
  const identity = releaseIdentity(
    process.env,
    execFileSync("git", ["rev-parse", "HEAD"], { encoding: "utf8" }).trim(),
  );
  if (operation === "pack" && archive && expectedDigest === undefined) {
    const sha256 = await packReleaseAssets(process.cwd(), resolve(archive), identity);
    if (!process.env.GITHUB_OUTPUT) throw new Error("Missing release asset output channel");
    appendFileSync(process.env.GITHUB_OUTPUT, `sha256=${sha256}\n`);
  } else if (operation === "extract" && archive && expectedDigest) {
    await extractReleaseAssets(process.cwd(), resolve(archive), expectedDigest, identity);
  } else {
    throw new Error("Usage: release-assets.ts pack <archive> | extract <archive> <sha256>");
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  await main();
}
