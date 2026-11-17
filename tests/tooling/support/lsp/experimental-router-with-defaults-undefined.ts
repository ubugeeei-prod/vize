import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

import {
  currentDefaultNativeType,
  previousDefaultNativeType,
} from "../../../../tools/support/compat/content-mapper/with-defaults-undefined-transition.ts";

type GoldenEntry = { file: string; sha256: string };
type Identity = { file: string; sha256: string; bytes: number };

const receiptSha256 = "b364b088f8783098bde0a5fa2d8c51b589881ed24fad963caf6d5d157b92f232";
const sha256 = (bytes: Buffer | string) => createHash("sha256").update(bytes).digest("hex");

/**
 * The #13179 successor of a generated golden: `undefined` joins
 * `$DefaultNativeType`, and every generated map coordinate after it moves by
 * the inserted bytes. Nothing else may differ.
 */
export function applyWithDefaultsUndefined(previous: Buffer): Buffer {
  const text = previous.toString("utf8");
  assert.equal(text.split(previousDefaultNativeType).length, 2, "one #13179 predecessor line");
  assert.equal(text.includes(currentDefaultNativeType), false);
  const at = Buffer.byteLength(text.slice(0, text.indexOf(previousDefaultNativeType)));
  const delta = Buffer.byteLength(currentDefaultNativeType) - Buffer.byteLength(previousDefaultNativeType);
  const shift = (offset: string) => String(Number(offset) >= at ? Number(offset) + delta : offset);
  const current = text
    .replace(previousDefaultNativeType, () => currentDefaultNativeType)
    .replace(
      /@vize-map: (\d+):(\d+) -> /gu,
      (_, start: string, end: string) => `@vize-map: ${shift(start)}:${shift(end)} -> `,
    );
  return Buffer.from(current);
}

/**
 * Authenticate a #13179 golden as its shared usage gate predecessor plus the
 * certified `$DefaultNativeType` edit, and return that predecessor.
 */
export function withDefaultsUndefinedPredecessor(
  custody: string,
  entry: GoldenEntry,
  bytes: Buffer,
): { entry: GoldenEntry; bytes: Buffer } {
  if (!entry.file.includes(".with-defaults-undefined")) return { entry, bytes };
  const receiptBytes = fs.readFileSync(path.join(custody, "with-defaults-undefined-20261117.json"));
  assert.equal(sha256(receiptBytes), receiptSha256);
  const receipt = JSON.parse(receiptBytes.toString("utf8"));
  assert.equal(receipt.schema, "vize.experimental-router-with-defaults-undefined-transition.v1");
  assert.equal(receipt.issue, 13179);
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
  assert.deepEqual(applyWithDefaultsUndefined(previous), bytes);
  return {
    entry: { file: transition.previous.file, sha256: transition.previous.sha256 },
    bytes: previous,
  };
}
