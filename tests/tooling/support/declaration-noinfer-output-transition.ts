import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

import { repoRoot } from "../_helpers/tool-commands.ts";

type Identity = { sha256: string; bytes: number };
type Output = {
  path: string;
  before: Identity;
  after: Identity;
  edits: Array<{ before: string; after: string }>;
};

const receiptPath =
  "tests/_fixtures/differential/typechecker/declaration-slot-export-own-instance/constructor-noinfer-successor.json";
const receiptBytes = fs.readFileSync(path.join(repoRoot, receiptPath));
const identity = (bytes: Buffer | string): Identity => ({
  sha256: createHash("sha256").update(bytes).digest("hex"),
  bytes: Buffer.byteLength(bytes),
});
assert.equal(
  identity(receiptBytes).sha256,
  "caa7512f3ae2869ef054db8d07ce2a8b7ab6c8a89fa53ba798df3e2b4ebfec72",
);
const receipt = JSON.parse(receiptBytes.toString("utf8")) as {
  schema: string;
  pullRequest: number;
  qualificationCredit: boolean;
  sourceReceipt: { path: string; sha256: string };
  outputs: Output[];
};
assert.equal(receipt.schema, "vize.declaration-constructor-noinfer-successor.v1");
assert.equal(receipt.pullRequest, 12523);
assert.equal(receipt.qualificationCredit, false);
assert.equal(
  identity(fs.readFileSync(path.join(repoRoot, receipt.sourceReceipt.path))).sha256,
  receipt.sourceReceipt.sha256,
);

/**
 * The NoInfer constructor (#12523) successor of a slot-reexport declaration
 * snapshot: the constructor return and the helper imports it names. `inverse`
 * restores the exact predecessor; `forward` replays it.
 */
export function noinferOutputTransition(current: Buffer, direction: "forward" | "inverse"): Buffer {
  const [from, to] =
    direction === "forward" ? (["before", "after"] as const) : (["after", "before"] as const);
  const output = receipt.outputs.find((entry) =>
    isDeepStrictEqualIdentity(identity(current), entry[from]),
  );
  assert.ok(output, "a recorded NoInfer declaration output");
  let text = current.toString("utf8");
  const edits = direction === "forward" ? output.edits : [...output.edits].reverse();
  for (const edit of edits) {
    assert.equal(text.split(edit[from]).length - 1, 1);
    assert.equal(text.split(edit[to]).length - 1, 0);
    text = text.replace(edit[from], () => edit[to]);
  }
  const changed = Buffer.from(text);
  assert.deepEqual(identity(changed), output[to]);
  return changed;
}

function isDeepStrictEqualIdentity(left: Identity, right: Identity): boolean {
  return left.sha256 === right.sha256 && left.bytes === right.bytes;
}
