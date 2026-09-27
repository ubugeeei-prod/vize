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
  return {
    workspaceRoot: resolve(cwd),
    sha: git("rev-parse", "HEAD"),
    tree: git("rev-parse", "HEAD^{tree}"),
  };
}

export async function createArchiveReceipt(archive, { cwd, nextestVersion }) {
  return {
    schemaVersion: 1,
    ...checkout(cwd),
    platform: process.platform,
    arch: process.arch,
    nextestVersion,
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
    nextestVersion: execFileSync("cargo", ["nextest", "--version"], { encoding: "utf8" }).trim(),
  };
  if (operation === "stamp") {
    writeFileSync(receiptPath, `${JSON.stringify(await createArchiveReceipt(archive, context))}\n`);
  } else {
    await verifyArchiveReceipt(JSON.parse(readFileSync(receiptPath, "utf8")), archive, context);
  }
}
