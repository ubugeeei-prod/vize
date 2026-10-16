import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { createReadStream, readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

async function digest(path) {
  const hash = createHash("sha256");
  for await (const chunk of createReadStream(path)) hash.update(chunk);
  return hash.digest("hex");
}

function checkout(cwd) {
  const git = (...args) => execFileSync("git", args, { cwd, encoding: "utf8" }).trim();
  const sha = git("rev-parse", "HEAD");
  const source = git("show", `${sha}:rust-toolchain.toml`);
  const channels = [...source.matchAll(/^channel\s*=\s*"([^"]+)"\s*$/gm)];
  if (channels.length !== 1 || !["1.99.0", "1.99.1"].includes(channels[0][1]))
    throw new Error("Rust archive requires a reviewed committed source toolchain");
  return {
    toolchain: channels[0][1],
    workspaceRoot: resolve(cwd),
    sha,
    tree: git("rev-parse", "HEAD^{tree}"),
  };
}

export async function createArchiveReceipt(
  archive,
  { cwd, nextestVersion, rustcVersion, env = process.env },
) {
  const requireTsgo = Object.hasOwn(env, "VIZE_TEST_REQUIRE_TSGO")
    ? env.VIZE_TEST_REQUIRE_TSGO
    : null;
  const disableTsgo = Object.hasOwn(env, "VIZE_TEST_DISABLE_TSGO")
    ? env.VIZE_TEST_DISABLE_TSGO
    : null;
  const { toolchain, ...source } = checkout(cwd);
  if (
    !rustcVersion?.startsWith(`rustc ${toolchain} `) ||
    !nextestVersion?.includes("0.9.146") ||
    !(
      (requireTsgo === "1" && disableTsgo === null) ||
      (requireTsgo === null && disableTsgo === "1")
    ) ||
    env.VIZE_NUXT_CONFIG_ITERATIONS !== "100"
  )
    throw new Error("Rust archive requires the pinned CI toolchain and TSGO runtime envelope");
  return {
    schemaVersion: 3,
    ...source,
    platform: process.platform,
    arch: process.arch,
    // cargo-binstall may select GNU or musl binaries on identical runners.
    // Their version banners differ, but the pinned nextest release is the
    // compatibility boundary for an archive consumed by another job.
    nextestVersion: "0.9.146",
    rustcVersion,
    cargoProfile: "ci",
    requireTsgo,
    disableTsgo,
    nuxtIterations: env.VIZE_NUXT_CONFIG_ITERATIONS || "",
    archiveSha256: await digest(archive),
  };
}

export async function verifyArchiveReceipt(receipt, archive, context) {
  const actual = await createArchiveReceipt(archive, context);
  for (const key of Object.keys(actual)) {
    if (receipt[key] !== actual[key]) throw new Error(`Rust test archive ${key} mismatch`);
  }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [operation, archive, receiptPath] = process.argv.slice(2);
  if (!archive || !receiptPath || !["stamp", "verify"].includes(operation)) {
    throw new Error("usage: rust-test-archive.mjs stamp|verify ARCHIVE RECEIPT");
  }
  const context = {
    cwd: process.cwd(),
    rustcVersion: execFileSync("rustc", ["-Vv"], { encoding: "utf8" }).trim(),
    nextestVersion: execFileSync("cargo", ["nextest", "--version"], { encoding: "utf8" }).trim(),
  };
  if (operation === "stamp") {
    writeFileSync(receiptPath, `${JSON.stringify(await createArchiveReceipt(archive, context))}\n`);
  } else {
    await verifyArchiveReceipt(JSON.parse(readFileSync(receiptPath, "utf8")), archive, context);
  }
}
