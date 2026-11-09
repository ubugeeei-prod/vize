import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

type GoldenEntry = { file: string; sha256: string };
type Identity = { file: string; sha256: string; bytes: number };
type Edit = { before: string; after: string };

const receiptSha256 = "a2aed604a93359368ba76de101a7a7925d9c1e5d4e8216237bd782f515d45e2e";
const sha256 = (bytes: Buffer | string) => createHash("sha256").update(bytes).digest("hex");
const mapOffsets = (line: string) =>
  (/@vize-map: (\d+):(\d+) -> /u.exec(line) ?? []).slice(1).map(Number);

/**
 * Authenticate a #11654 golden as its canon-leftovers predecessor plus the
 * recorded helper line and its shifted setup map line, and return that
 * predecessor for the older chain.
 */
export function instanceInputKeysPredecessor(
  custody: string,
  repoRoot: string,
  entry: GoldenEntry,
  bytes: Buffer,
): { entry: GoldenEntry; bytes: Buffer } | null {
  if (!entry.file.includes(".instance-input-keys")) return null;
  const receiptBytes = fs.readFileSync(path.join(custody, "instance-input-keys-20261109.json"));
  assert.equal(sha256(receiptBytes), receiptSha256);
  const receipt = JSON.parse(receiptBytes.toString("utf8"));
  assert.equal(receipt.schema, "vize.experimental-router-instance-input-keys-transition.v1");
  assert.equal(receipt.issue, 11654);
  assert.equal(receipt.qualificationCredit, false);
  const source = fs.readFileSync(path.join(repoRoot, receipt.sourceReceipt.path));
  assert.equal(sha256(source), receipt.sourceReceipt.sha256);
  assert.deepEqual(receipt.edit, JSON.parse(source.toString("utf8")).files[0].hunks[0]);
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
  const edits: Edit[] = [transition.mapEdit, receipt.edit];
  let restored = bytes.toString("utf8");
  for (const edit of edits) {
    assert.equal(restored.split(edit.after).length, 2);
    restored = restored.replace(edit.after, () => edit.before);
  }
  assert.equal(restored, previous.toString("utf8"));
  // Every generated offset after the helper moves by the helper's growth.
  const shift = Buffer.byteLength(receipt.edit.after) - Buffer.byteLength(receipt.edit.before);
  assert.deepEqual(
    mapOffsets(transition.mapEdit.after),
    mapOffsets(transition.mapEdit.before).map((offset) => offset + shift),
  );
  for (const moved of Object.values(receipt.completionDataPositions) as Array<{
    before: number;
    after: number;
  }>)
    assert.equal(moved.after - moved.before, shift);
  return {
    entry: { file: transition.previous.file, sha256: transition.previous.sha256 },
    bytes: previous,
  };
}
