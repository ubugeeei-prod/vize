import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

type GoldenEntry = { file: string; sha256: string };
type Transition = { currentSha256: string };

export function assertCanonLeftoversGolden(
  custody: string,
  repoRoot: string,
  entry: GoldenEntry,
  transitionFile: string,
  strictTransition: Transition,
  bytes: Buffer,
): boolean {
  if (!entry.file.includes(".canon-leftovers")) return false;
  const receiptBytes = fs.readFileSync(path.join(custody, "canon-leftovers-20261108.json"));
  assert.equal(
    createHash("sha256").update(receiptBytes).digest("hex"),
    "5c66161ef05392a924e083980cec1dcb9b0de8f0d8bac4fd17b6402b0c05f43f",
  );
  const receipt = JSON.parse(receiptBytes.toString("utf8"));
  assert.equal(receipt.schema, "vize.experimental-router-canon-leftovers-transition.v1");
  assert.equal(receipt.issue, 11499);
  assert.equal(receipt.pullRequest, 12162);
  assert.equal(receipt.qualificationCredit, false);
  const sourceReceipt = fs.readFileSync(path.join(repoRoot, receipt.sourceReceipt.path));
  assert.equal(
    createHash("sha256").update(sourceReceipt).digest("hex"),
    receipt.sourceReceipt.sha256,
  );
  const transition = receipt.entries.find(
    (row: { current: { file: string } }) => row.current.file === entry.file,
  );
  assert.ok(transition);
  assert.equal(transition.previous.file, transitionFile);
  assert.equal(transition.previous.sha256, strictTransition.currentSha256);
  assert.equal(transition.current.sha256, entry.sha256);
  const previousBytes = fs.readFileSync(path.join(custody, transition.previous.file));
  assert.equal(previousBytes.length, transition.previous.bytes);
  assert.equal(
    createHash("sha256").update(previousBytes).digest("hex"),
    transition.previous.sha256,
  );
  assert.equal(bytes.length, transition.current.bytes);
  return true;
}
