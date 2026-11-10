import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

import { verbatimSetupRelocation } from "../../../../tools/support/compat/content-mapper/verbatim-setup-relocation.ts";

type GoldenEntry = { file: string; sha256: string };
type Identity = { file: string; sha256: string; bytes: number };

const receiptSha256 = "6cd5b174e5dbe5662a4700e9042aac26191efd97eb43d6524d2cbb1e8897f0a6";
const sha256 = (bytes: Buffer | string) => createHash("sha256").update(bytes).digest("hex");

/**
 * Authenticate a NoInfer constructor golden (#12523) as its predecessor with
 * the old constructor return and the verbatim setup lines (#12550) undone, and
 * return that predecessor. The constructor edit sits after every recorded map
 * coordinate; only the setup lines and their map comment move.
 */
export function constructorNoInferPredecessor(
  custody: string,
  repoRoot: string,
  entry: GoldenEntry,
  bytes: Buffer,
): { entry: GoldenEntry; bytes: Buffer } {
  if (!entry.file.includes(".constructor-noinfer")) return { entry, bytes };
  const receiptBytes = fs.readFileSync(path.join(custody, "constructor-noinfer-20261110.json"));
  assert.equal(sha256(receiptBytes), receiptSha256);
  const receipt = JSON.parse(receiptBytes.toString("utf8"));
  assert.equal(receipt.schema, "vize.experimental-router-constructor-noinfer-transition.v1");
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
  const current = bytes.toString("utf8");
  assert.equal(current.split(receipt.edit.after).length, 2);
  assert.deepEqual(receipt.setupVerbatim, {
    pullRequest: 12550,
    certifier: "tools/support/compat/content-mapper/verbatim-setup-relocation.ts",
  });
  const restored = current.replace(receipt.edit.after, () => receipt.edit.before);
  verbatimSetupRelocation(previous, Buffer.from(restored));
  const cut = Buffer.byteLength(current.slice(0, current.indexOf(receipt.edit.after)));
  for (const match of current.matchAll(/@vize-map: (\d+):(\d+) -> /gu))
    assert.ok(Number(match[2]) <= cut, "recorded map coordinates precede the edit");
  return {
    entry: { file: transition.previous.file, sha256: transition.previous.sha256 },
    bytes: previous,
  };
}
