import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

import { optionalJsxWitnessHelperEdit } from "../../../../tools/support/compat/content-mapper/optional-jsx-witness-transition.ts";

type GoldenEntry = { file: string; sha256: string };
type Identity = { file: string; sha256: string; bytes: number };

const receiptSha256 = "ed93f7bf778e15d9bcf07129b27e8d0f44488714e28fd53371599b592d20c687";
const sha256 = (bytes: Buffer | string) => createHash("sha256").update(bytes).digest("hex");

/**
 * The successor of a generated golden whose `$AuthoredKeyWitness` maps its
 * keys as optional: every generated map coordinate after the helper line moves
 * by the inserted bytes. Nothing else may differ.
 */
export function applyOptionalJsxWitness(previous: Buffer): Buffer {
  const edit = optionalJsxWitnessHelperEdit();
  const text = previous.toString("utf8");
  assert.equal(text.split(edit.before).length, 2, "one optional JSX witness predecessor line");
  assert.equal(text.includes(edit.after), false);
  const at = Buffer.byteLength(text.slice(0, text.indexOf(edit.before)));
  const delta = Buffer.byteLength(edit.after) - Buffer.byteLength(edit.before);
  const shift = (offset: string) => String(Number(offset) >= at ? Number(offset) + delta : offset);
  const current = text
    .replace(edit.before, () => edit.after)
    .replace(
      /@vize-map: (\d+):(\d+) -> /gu,
      (_, start: string, end: string) => `@vize-map: ${shift(start)}:${shift(end)} -> `,
    );
  return Buffer.from(current);
}

/**
 * Authenticate an optional JSX witness golden as its `withDefaults` predecessor
 * plus the certified helper edit, and return that predecessor.
 */
export function optionalJsxWitnessPredecessor(
  custody: string,
  entry: GoldenEntry,
  bytes: Buffer,
): { entry: GoldenEntry; bytes: Buffer } {
  if (!entry.file.includes(".optional-jsx-witness")) return { entry, bytes };
  const receiptBytes = fs.readFileSync(path.join(custody, "optional-jsx-witness.json"));
  assert.equal(sha256(receiptBytes), receiptSha256);
  const receipt = JSON.parse(receiptBytes.toString("utf8"));
  assert.equal(receipt.schema, "vize.experimental-router-optional-jsx-witness-transition.v1");
  assert.equal(receipt.qualificationCredit, false);
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
  assert.deepEqual(applyOptionalJsxWitness(previous), bytes);
  // Completion data positions sit after the helper and move by its growth.
  const edit = optionalJsxWitnessHelperEdit();
  const shift = Buffer.byteLength(edit.after) - Buffer.byteLength(edit.before);
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
