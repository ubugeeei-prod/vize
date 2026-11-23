import assert from "node:assert/strict";
import { execFileSync, spawn } from "node:child_process";
import { createHash } from "node:crypto";
import { fileURLToPath, pathToFileURL } from "node:url";
import { gitBodyDigest as retainedGitBodyDigest } from "./warm-type-backed-path-host.ts";

/** Hash complete regular Git blobs; preserve absence and reject mode/type changes. */
export function gitBodyDigest(root: string, revision: string, file: string): string | null {
  assert.match(revision, /^[a-f0-9]{40}$/u);
  const entry = execFileSync(
    "git",
    ["--no-replace-objects", "ls-tree", "-l", "-z", revision, "--", file],
    {
      cwd: root,
    },
  );
  if (entry.length === 0) return null;
  const record = entry.toString("utf8");
  assert.ok(Buffer.from(record).equals(entry));
  const records = record.split("\0");
  assert.equal(records.length, 2);
  assert.equal(records[1], "");
  const match = /^100644 blob ([a-f0-9]{40}) +([0-9]+)\t(.+)$/u.exec(records[0]);
  assert.ok(match, `one regular source blob is required: ${file}`);
  assert.equal(match[3], file);
  const size = Number(match[2]);
  assert(Number.isSafeInteger(size) && size >= 0);
  if (size > 1024 * 1024) {
    const worker = fileURLToPath(import.meta.url);
    const digest = execFileSync(process.execPath, [worker, match[1], String(size)], {
      cwd: root,
    }).toString("utf8");
    assert.match(digest, /^[a-f0-9]{64}$/u);
    return digest;
  }
  return retainedGitBodyDigest(root, revision, file);
}

function streamBlob(oid: string, size: number) {
  const child = spawn("git", ["--no-replace-objects", "cat-file", "blob", oid], {
    stdio: ["ignore", "pipe", "inherit"],
  });
  const hash = createHash("sha256");
  let bytes = 0;
  child.stdout.on("data", (body: Buffer) => {
    bytes += body.length;
    hash.update(body);
  });
  child.on("error", (error) => {
    console.error(error);
    process.exitCode = 1;
  });
  child.on("close", (status, signal) => {
    if (signal) {
      process.kill(process.pid, signal);
      return;
    }
    if (status !== 0) {
      process.exitCode = status ?? 1;
      return;
    }
    if (bytes !== size) {
      console.error("Incomplete Git blob");
      process.exitCode = 1;
      return;
    }
    process.stdout.write(hash.digest("hex"));
  });
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  streamBlob(process.argv[2], Number(process.argv[3]));
}
