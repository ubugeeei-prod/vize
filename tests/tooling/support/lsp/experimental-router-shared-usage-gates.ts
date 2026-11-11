import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

import { certifyFallthroughSourceFields } from "../../../../tools/support/compat/content-mapper/shared-usage-gates-transition.ts";

type GoldenEntry = { file: string; sha256: string };
type Identity = { file: string; sha256: string; bytes: number };

const receiptSha256 = "96de8f9b799ea1450bb3745ccbfef14c6af889ddc8781468938ce141cd4f1a4e";
const sha256 = (bytes: Buffer | string) => createHash("sha256").update(bytes).digest("hex");

/**
 * Authenticate a shared usage gate golden (#12793) as its constructor NoInfer
 * predecessor plus the one certified raw fallthrough source field, and return
 * that predecessor. The field sits after every recorded map coordinate.
 */
export function sharedUsageGatesPredecessor(
  custody: string,
  repoRoot: string,
  entry: GoldenEntry,
  bytes: Buffer,
): { entry: GoldenEntry; bytes: Buffer } {
  if (!entry.file.includes(".shared-usage-gates")) return { entry, bytes };
  const receiptBytes = fs.readFileSync(path.join(custody, "shared-usage-gates-20261111.json"));
  assert.equal(sha256(receiptBytes), receiptSha256);
  const receipt = JSON.parse(receiptBytes.toString("utf8"));
  assert.equal(receipt.schema, "vize.experimental-router-shared-usage-gates-transition.v1");
  assert.equal(receipt.pullRequest, 12793);
  assert.equal(receipt.qualificationCredit, false);
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
  const [field, ...extra] = certifyFallthroughSourceFields(previous, bytes);
  assert.ok(field && extra.length === 0, "one raw fallthrough source field");
  assert.equal(field.replacement.toString("utf8"), receipt.field);
  assert.deepEqual(
    Buffer.concat([
      previous.subarray(0, field.start),
      field.replacement,
      previous.subarray(field.end),
    ]),
    bytes,
  );
  for (const match of bytes.toString("utf8").matchAll(/@vize-map: (\d+):(\d+) -> /gu))
    assert.ok(Number(match[2]) <= field.at, "recorded map coordinates precede the field");
  return {
    entry: { file: transition.previous.file, sha256: transition.previous.sha256 },
    bytes: previous,
  };
}
