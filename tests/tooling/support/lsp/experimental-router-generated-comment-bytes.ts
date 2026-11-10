import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

import { generatedCommentBytesCut } from "../../../../tools/support/compat/content-mapper/generated-comment-bytes-cut.ts";

type GoldenEntry = { file: string; sha256: string };
type Identity = { file: string; sha256: string; bytes: number };

const receiptSha256 = "3e75da62628885a9e43e52f7a8f730c2d8988f83533c8e05d9b37f63c3cac85c";
const sha256 = (bytes: Buffer | string) => createHash("sha256").update(bytes).digest("hex");

/**
 * Authenticate a generated-comment-bytes golden as the exact comment cut of
 * its NoInfer predecessor, and return that predecessor: the debug-only map
 * lines and the `@ts-ignore` reasons are the only bytes that leave.
 */
export function generatedCommentBytesPredecessor(
  custody: string,
  repoRoot: string,
  entry: GoldenEntry,
  bytes: Buffer,
): { entry: GoldenEntry; bytes: Buffer } {
  if (!entry.file.includes(".generated-comment-bytes")) return { entry, bytes };
  const receiptBytes = fs.readFileSync(path.join(custody, "generated-comment-bytes-20261110.json"));
  assert.equal(sha256(receiptBytes), receiptSha256);
  const receipt = JSON.parse(receiptBytes.toString("utf8"));
  assert.equal(receipt.schema, "vize.experimental-router-generated-comment-bytes-transition.v1");
  assert.equal(receipt.qualificationCredit, false);
  assert.equal(
    receipt.certifier,
    "tools/support/compat/content-mapper/generated-comment-bytes-cut.ts",
  );
  const source = fs.readFileSync(path.join(repoRoot, receipt.sourceReceipt.path));
  assert.equal(sha256(source), receipt.sourceReceipt.sha256);
  const transition = receipt.entries.find(
    (row: { current: Identity }) => row.current.file === entry.file,
  );
  assert.ok(transition);
  assert.equal(transition.current.sha256, entry.sha256);
  assert.deepEqual(
    { sha256: sha256(bytes), bytes: bytes.length },
    { sha256: transition.current.sha256, bytes: transition.current.bytes },
  );
  const previous = fs.readFileSync(path.join(custody, transition.previous.file));
  assert.deepEqual(
    { sha256: sha256(previous), bytes: previous.length },
    { sha256: transition.previous.sha256, bytes: transition.previous.bytes },
  );
  assert.equal(
    generatedCommentBytesCut(previous.toString("utf8")).text,
    bytes.toString("utf8"),
    "only the generated comment cut separates the golden from its predecessor",
  );
  return {
    entry: { file: transition.previous.file, sha256: transition.previous.sha256 },
    bytes: previous,
  };
}
